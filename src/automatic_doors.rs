//! Automatic Doors: open nearby doors for real players, close only mod-opened doors.
//! All block mutations happen on the authoritative server's tick, with bounded scans.
use crate::settings::{amount, flag, on};
use mod_api::{
    BlockUpdateFlags, LevelRef, ServerHost, ServerTickEvent, ServerTickPhase,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_TRACKED: usize = 256;
const MAX_PLAYERS: usize = 24;
const MAX_CLOSES: usize = 32;
const SCAN_EVERY: u64 = 4;
const NEAR_SQUARED: f64 = 9.0;

#[derive(Clone)]
struct Opened {
    dimension: String,
    pos: [i32; 3],
    block: String,
    due: u64,
}

#[derive(Clone)]
struct Nearby {
    dimension: String,
    pos: [f64; 3],
    sneaking: bool,
}

#[derive(Default)]
pub struct AutomaticDoors {
    tick: AtomicU64,
    opened: Mutex<Vec<Opened>>,
}

fn property<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let (_, properties) = state.split_once('[')?;
    properties.strip_suffix(']')?.split(',').find_map(|entry| {
        let (name, value) = entry.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn changed_open(state: &str, open: bool) -> Option<String> {
    let (name, values) = state.split_once('[')?;
    let values = values.strip_suffix(']')?;
    let mut found = false;
    let result: Vec<String> = values.split(',').map(|entry| {
        if entry.split_once('=').is_some_and(|(key, _)| key == "open") {
            found = true;
            format!("open={}", if open { "true" } else { "false" })
        } else {
            entry.to_owned()
        }
    }).collect();
    found.then(|| format!("{name}[{}]", result.join(",")))
}

fn door_name(state: &str, iron: bool) -> Option<&str> {
    let name = state.split_once('[')?.0;
    if !name.ends_with("_door") || name.ends_with("_trapdoor") {
        return None;
    }
    if !iron && name == "minecraft:iron_door" {
        return None;
    }
    Some(name)
}

fn close_enough(a: [f64; 3], block: [i32; 3]) -> bool {
    let dx = a[0] - (f64::from(block[0]) + 0.5);
    let dy = a[1] - f64::from(block[1]);
    let dz = a[2] - (f64::from(block[2]) + 0.5);
    dx * dx + dy * dy + dz * dz < NEAR_SQUARED
}

fn delay_ticks(milliseconds: f64) -> u64 {
    (milliseconds.clamp(0.0, 10000.0) / 50.0).ceil() as u64
}

fn lower_pos(state: &str, pos: [i32; 3]) -> Option<[i32; 3]> {
    let mut lower = pos;
    if property(state, "half") == Some("upper") {
        lower[1] = lower[1].checked_sub(1)?;
    }
    Some(lower)
}

fn flip(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3],
        expected: &str, open: bool, iron: bool) -> bool {
    let Some(state) = level.block_state(host, pos) else { return false; };
    let Some(text) = host.block_state_string(state) else { return false; };
    if door_name(&text, iron) != Some(expected)
        || property(&text, "half") != Some("lower")
        || property(&text, "open") != Some(if open { "false" } else { "true" })
        || property(&text, "powered") == Some("true") {
        return false;
    }
    let Some(next) = changed_open(&text, open).and_then(|text| host.block_state_id(&text)) else {
        return false;
    };
    if level.set_block(host, pos, next, BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS)
        != Ok(true) {
        return false;
    }
    // Preserve the two-block door's state even when the engine hasn't propagated it yet.
    if let Some(y) = pos[1].checked_add(1) {
        let upper = [pos[0], y, pos[2]];
        if let Some(state) = level.block_state(host, upper) {
            if let Some(text) = host.block_state_string(state) {
                if door_name(&text, iron) == Some(expected)
                    && property(&text, "half") == Some("upper")
                    && property(&text, "open") != Some(if open { "true" } else { "false" }) {
                    if let Some(next) = changed_open(&text, open)
                        .and_then(|text| host.block_state_id(&text)) {
                        let _ = level.set_block(host, upper, next, BlockUpdateFlags::CLIENTS);
                    }
                }
            }
        }
    }
    true
}

impl AutomaticDoors {
    fn players(&self, host: &dyn ServerHost) -> Vec<Nearby> {
        host.players().server_players().into_iter().filter_map(|player| {
            if player.is_spectator(host) != Some(false) { return None; }
            let level = player.level(host)?;
            let dimension = level.dimension().id(host)?;
            let pos = player.entity().position(host)?;
            if !pos.iter().all(|axis| axis.is_finite()) { return None; }
            let sneaking = player.is_secondary_use_active(host).unwrap_or(false);
            Some(Nearby { dimension, pos, sneaking })
        }).collect()
    }

    fn close_due(&self, host: &mut dyn ServerHost, players: &[Nearby], now: u64, iron: bool) {
        let due = self.opened.lock().ok().map(|list| {
            list.iter().filter(|entry| entry.due <= now)
                .take(MAX_CLOSES).cloned().collect::<Vec<_>>()
        }).unwrap_or_default();
        for entry in due {
            if players.iter().any(|player| player.dimension == entry.dimension
                && close_enough(player.pos, entry.pos)) {
                continue;
            }
            let Some(level) = host.level(&entry.dimension) else { continue; };
            if !level.in_place(host) { continue; }
            let Some(state) = level.block_state(host, entry.pos) else { continue; };
            let Some(text) = host.block_state_string(state) else { continue; };
            // A player may have closed or removed this door after we opened it.
            if door_name(&text, iron) == Some(entry.block.as_str())
                && property(&text, "open") == Some("true")
                && property(&text, "powered") != Some("true") {
                let _ = flip(host, level, entry.pos, &entry.block, false, iron);
            }
            if let Ok(mut list) = self.opened.lock() {
                list.retain(|item| item.dimension != entry.dimension || item.pos != entry.pos);
            }
        }
    }

    fn try_open(&self, host: &mut dyn ServerHost, level: LevelRef,
                dimension: &str, pos: [i32; 3], due: u64, iron: bool) {
        let Some(state) = level.block_state(host, pos) else { return; };
        let Some(text) = host.block_state_string(state) else { return; };
        let Some(name) = door_name(&text, iron) else { return; };
        let Some(lower) = lower_pos(&text, pos) else { return; };
        let Some(state) = level.block_state(host, lower) else { return; };
        let Some(lower_text) = host.block_state_string(state) else { return; };
        if property(&lower_text, "open") != Some("false")
            || property(&lower_text, "powered") == Some("true") {
            return;
        }
        if self.opened.lock().ok().is_none_or(|list| {
            list.len() >= MAX_TRACKED
                || list.iter().any(|item| item.dimension == dimension && item.pos == lower)
        }) {
            return;
        }
        if flip(host, level, lower, name, true, iron) {
            if let Ok(mut list) = self.opened.lock() {
                list.push(Opened {
                    dimension: dimension.to_owned(), pos: lower,
                    block: name.to_owned(), due,
                });
            }
        }
    }

    fn open_nearby(&self, host: &mut dyn ServerHost,
                   players: &[Nearby], now: u64, delay: u64, iron: bool) {
        let prevent_sneak = flag(host, "auto_prevent_sneak", true);
        let count = players.len();
        if count == 0 { return; }
        let start = (now / SCAN_EVERY) as usize % count;
        for index in 0..count.min(MAX_PLAYERS) {
            let player = &players[(start + index) % count];
            if prevent_sneak && player.sneaking { continue; }
            let Some(level) = host.level(&player.dimension) else { continue; };
            if !level.in_place(host) { continue; }
            let [x, y, z] = player.pos.map(|part| part.floor() as i32);
            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    for dz in -1_i32..=1 {
                        let (Some(px), Some(py), Some(pz)) =
                            (x.checked_add(dx), y.checked_add(dy), z.checked_add(dz)) else {
                            continue;
                        };
                        let pos = [px, py, pz];
                        if close_enough(player.pos, pos) {
                            self.try_open(host, level, &player.dimension, pos,
                                          now.saturating_add(delay), iron);
                        }
                    }
                }
            }
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let now = self.tick.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        let delay = delay_ticks(amount(host, "auto_door_open_time", 2500., 0., 10000.));
        if !on(host, "automatic_doors") || delay == 0 {
            if let Ok(mut list) = self.opened.lock() { list.clear(); }
            return;
        }
        let iron = flag(host, "auto_iron_doors", true);
        let players = self.players(host);
        self.close_due(host, &players, now, iron);
        if now % SCAN_EVERY == 0 {
            self.open_nearby(host, &players, now, delay, iron);
        }
    }

    pub fn reset(&self) {
        self.tick.store(0, Ordering::Relaxed);
        if let Ok(mut list) = self.opened.lock() { list.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn door_classification_and_states() {
        assert_eq!(door_name("minecraft:oak_door[half=lower,open=false]", true),
                   Some("minecraft:oak_door"));
        assert_eq!(door_name("minecraft:iron_door[open=false]", false), None);
        assert_eq!(door_name("minecraft:birch_trapdoor[open=false]", true), None);
        assert_eq!(door_name("minecraft:oak_fence_gate[open=false]", true), None);
        assert_eq!(door_name("minecraft:stone", true), None);
        let original = "minecraft:oak_door[half=lower,open=false,powered=false]";
        assert_eq!(changed_open(original, true).as_deref(),
                   Some("minecraft:oak_door[half=lower,open=true,powered=false]"));
        assert_eq!(lower_pos("oak_door[half=upper]", [10, 71, 10]), Some([10, 70, 10]));
    }

    #[test]
    fn delay_and_proximity_are_bounded() {
        assert_eq!(delay_ticks(2500.), 50);
        assert_eq!(delay_ticks(10000.), 200);
        assert_eq!(delay_ticks(0.), 0);
        assert!(close_enough([1.5, 64., 1.5], [1, 64, 1]));
        assert!(!close_enough([10.5, 64., 10.5], [1, 64, 1]));
    }
}
