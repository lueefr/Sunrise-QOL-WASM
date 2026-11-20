//! Automatic Doors: open nearby doors for real players, close only mod-opened doors.
//! Match the original per-player tick scan and whole-second close scheduling.
use crate::settings::{amount, flag, on};
use mod_api::{
    BlockUpdateFlags, LevelRef, ServerHost, ServerTickEvent, ServerTickPhase,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

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
    bounds: [[f64; 3]; 2],
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

fn close_enough(a: [f64; 3], bounds: [[f64; 3]; 2], block: [i32; 3]) -> bool {
    // The original checks an AABB around the doorway, then compares the
    // player's integer BlockPos at a strict radius of three blocks.
    // Entity queries intersect the player's full box, not just their feet.
    let radius = [2.0, 1.0, 2.0];
    let inside = (0..3).all(|axis| {
        bounds[1][axis] > f64::from(block[axis]) - radius[axis]
            && bounds[0][axis] < f64::from(block[axis]) + radius[axis]
    });
    let pos = a.map(|axis| axis.floor() as i32);
    let dx = f64::from(pos[0] - block[0]);
    let dy = f64::from(pos[1] - block[1]);
    let dz = f64::from(pos[2] - block[2]);
    inside && dx * dx + dy * dy + dz * dz < NEAR_SQUARED
}

fn delay_ticks(milliseconds: f64) -> u64 {
    // Serilum schedules (doorOpenTime / 1000) * 20 using integer division.
    ((milliseconds.clamp(0.0, 10000.0) as u64) / 1000) * 20
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
        {
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
            let bounds = player.entity().bounding_box(host)?;
            if !pos.iter().all(|axis| axis.is_finite()) { return None; }
            let sneaking = player.is_secondary_use_active(host).unwrap_or(false);
            Some(Nearby { dimension, pos, sneaking, bounds })
        }).collect()
    }

    fn close_due(&self, host: &mut dyn ServerHost, players: &[Nearby], now: u64, iron: bool) {
        let skip_sneak = flag(host, "auto_prevent_sneak", true);
        let due = self.opened.lock().ok().map(|list| {
            list.iter().filter(|entry| entry.due <= now)
                .cloned().collect::<Vec<_>>()
        }).unwrap_or_default();
        for entry in due {
            let upper = [entry.pos[0], entry.pos[1].saturating_add(1), entry.pos[2]];
            if players.iter().any(|player| player.dimension == entry.dimension
                && (!skip_sneak || !player.sneaking)
                && close_enough(player.pos, player.bounds, upper)) {
                continue;
            }
            let Some(level) = host.level(&entry.dimension) else { continue; };
            if !level.in_place(host) { continue; }
            let Some(state) = level.block_state(host, entry.pos) else { continue; };
            let Some(text) = host.block_state_string(state) else { continue; };
            if door_name(&text, iron) == Some(entry.block.as_str()) {
                // Upstream closes every supported door in the adjacent 3x3
                // square, including doors not initially opened by this module.
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        let pos = [upper[0] + dx, upper[1], upper[2] + dz];
                        let Some(state) = level.block_state(host, pos) else { continue; };
                        let Some(text) = host.block_state_string(state) else { continue; };
                        let Some(name) = door_name(&text, iron) else { continue; };
                        let Some(lower) = lower_pos(&text, pos) else { continue; };
                        let _ = flip(host, level, lower, name, false, iron);
                    }
                }
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
        let is_open = match property(&lower_text, "open") {
            Some("true") => true,
            Some("false") => false,
            _ => return,
        };
        if self.opened.lock().ok().is_none_or(|list| {
            list.iter().any(|item| item.dimension == dimension && item.pos == lower)
        }) {
            return;
        }
        // Upstream schedules closure even when the nearby door is already open.
        if is_open || flip(host, level, lower, name, true, iron) {
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
        // Original onPlayerTick: the 3x3 X/Z square on the block layer
        // directly above the player's feet, not a 3x3x3 proximity sphere.
        for player in players {
            if prevent_sneak && player.sneaking { continue; }
            let Some(level) = host.level(&player.dimension) else { continue; };
            if !level.in_place(host) { continue; }
            let [x, feet, z] = player.pos.map(|part| part.floor() as i32);
            let Some(y) = feet.checked_add(1) else { continue; };
            for dx in -1_i32..=1 {
                for dz in -1_i32..=1 {
                    let (Some(px), Some(pz)) =
                        (x.checked_add(dx), z.checked_add(dz)) else { continue; };
                    self.try_open(host, level, &player.dimension, [px, y, pz],
                                  now.saturating_add(delay), iron);
                }
            }
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let now = self.tick.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        let open_time = amount(host, "auto_door_open_time", 2500., 0., 10000.);
        let delay = delay_ticks(open_time);
        if !on(host, "automatic_doors") || open_time == 0. {
            if let Ok(mut list) = self.opened.lock() { list.clear(); }
            return;
        }
        let iron = flag(host, "auto_iron_doors", true);
        let players = self.players(host);
        self.close_due(host, &players, now, iron);
        self.open_nearby(host, &players, now, delay, iron);
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
        assert_eq!(delay_ticks(2500.), 40);
        assert_eq!(delay_ticks(999.), 0);
        assert_eq!(delay_ticks(1999.), 20);
        assert_eq!(delay_ticks(10000.), 200);
        assert_eq!(delay_ticks(0.), 0);
        assert_eq!(delay_ticks(500.), 0);
        assert!(close_enough([1.5, 64., 1.5], [[1.2, 64., 1.2], [1.8, 65.8, 1.8]], [1, 64, 1]));
        assert!(!close_enough([10.5, 64., 10.5], [[10.2, 64., 10.2], [10.8, 65.8, 10.8]], [1, 64, 1]));
    }

    #[test]
    fn closure_checks_box_overlap_and_strict_integer_distance() {
        let door = [0, 65, 0];
        // Feet are outside the query box, but the player's body intersects it.
        assert!(close_enough([2.2, 64., 0.5], [[1.9, 64., 0.2], [2.5, 65.8, 0.8]], door));
        // Merely touching the query boundary is not an AABB intersection.
        assert!(!close_enough([2.3, 64., 0.5], [[2., 64., 0.2], [2.6, 65.8, 0.8]], door));
        // AABB overlap alone is insufficient: integer BlockPos distance must be < 3.
        assert!(!close_enough([2.2, 64., 2.2], [[1.9, 64., 1.9], [2.5, 65.8, 2.5]], door));
    }
}
