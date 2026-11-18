//! Leaf decay uses bounded server-side work and the game's regular block operations.
//! Leaves Be Gone's short decay delay is combined with Leaves Us In Peace's matching rules.
//! The public sandbox API cannot replace vanilla's LeavesBlock.updateDistance mixin.
use mod_api::{BlockChangedEvent, LevelHost, LevelRef, ServerHost, ServerTickEvent, ServerTickPhase};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::settings::{amount, flag};

const MAX_TASKS: usize = 8192;
const MAX_VISITS: usize = 512;
const MAX_CHECKS: usize = 48;
const FACES: [[i32; 3]; 6] = [
    [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1],
];

#[derive(Clone)]
struct Task {
    dimension: String,
    pos: [i32; 3],
    due: u64,
    retries: u8,
}

#[derive(Default)]
pub struct Decay {
    ticks: AtomicU64,
    tasks: Mutex<VecDeque<Task>>,
}

fn nearby(pos: [i32; 3], offset: [i32; 3]) -> [i32; 3] {
    [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]]
}

fn is_leaf(name: &str) -> bool {
    name.ends_with("_leaves")
}

fn is_log(name: &str) -> bool {
    name.ends_with("_log") || name.ends_with("_stem")
        || name.ends_with("_wood") || name.ends_with("_hyphae")
        || name == "minecraft:mangrove_roots"
}

fn namespace(name: &str) -> (&str, &str) {
    name.split_once(':').unwrap_or(("minecraft", name))
}

fn species(name: &str) -> Option<&str> {
    let (_, path) = namespace(name);
    path.strip_suffix("_leaves")
        .or_else(|| path.strip_suffix("_log"))
        .or_else(|| path.strip_suffix("_wood"))
        .or_else(|| path.strip_suffix("_stem"))
        .or_else(|| path.strip_suffix("_hyphae"))
}

fn known_leaf(name: &str) -> bool {
    matches!(name,
        "minecraft:oak_leaves" | "minecraft:spruce_leaves" | "minecraft:birch_leaves"
        | "minecraft:jungle_leaves" | "minecraft:acacia_leaves"
        | "minecraft:dark_oak_leaves" | "minecraft:cherry_leaves"
        | "minecraft:mangrove_leaves" | "minecraft:pale_oak_leaves"
        | "minecraft:azalea_leaves" | "minecraft:flowering_azalea_leaves")
}

fn leaf_match(host: &dyn ServerHost, source: &str, other: &str, strict: bool) -> bool {
    if !strict || source == other { return true; }
    if matches!((source, other),
        ("minecraft:azalea_leaves", "minecraft:flowering_azalea_leaves")
        | ("minecraft:flowering_azalea_leaves", "minecraft:azalea_leaves")) {
        return true;
    }
    // Data packs may opt additional leaf types into each other's connectivity.
    let (ns, path) = namespace(source);
    let compatible = format!("leaves_us_in_peace:{ns}/{path}");
    if host.block_key(other).is_some_and(|key| host.block_in_tag(key, &compatible) == Some(true)) {
        return true;
    }
    let (ns, path) = namespace(other);
    let reciprocal = format!("leaves_us_in_peace:{ns}/{path}");
    if host.block_key(source).is_some_and(|key| host.block_in_tag(key, &reciprocal) == Some(true)) {
        return true;
    }
    !flag(host, "unknownLeavesMatchSelf", true)
        && (!known_leaf(source) || !known_leaf(other))
}

fn log_match(host: &dyn ServerHost, leaf: &str, log: &str, strict: bool) -> bool {
    if !strict { return true; }
    if (log.ends_with("_wood") || log.ends_with("_hyphae"))
        && !flag(host, "woodPreventsDecay", false) {
        return false;
    }
    // A tag named for the log can explicitly authorize leaf species.
    let (ns, path) = namespace(log);
    let tag = format!("leaves_us_in_peace:{ns}/{path}");
    if host.block_key(leaf).is_some_and(|key| host.block_in_tag(key, &tag) == Some(true)) {
        return true;
    }
    if !known_leaf(leaf) { return true; }
    if leaf == "minecraft:azalea_leaves" || leaf == "minecraft:flowering_azalea_leaves" {
        return log == "minecraft:oak_log" || log == "minecraft:oak_wood";
    }
    if leaf == "minecraft:mangrove_leaves" && log == "minecraft:mangrove_roots" {
        return true;
    }
    if namespace(log).0 != "minecraft" { return true; }
    species(leaf).is_some_and(|family| species(log) == Some(family))
}

// Only unloaded or otherwise unavailable positions leave the result unknown.
// A path of at most six leaf blocks to a matching log is required, as in vanilla.
fn supported(host: &dyn ServerHost, level: LevelRef, start: [i32; 3], source: &str) -> Option<bool> {
    let strict_leaves = flag(host, "matchLeafTypes", true);
    let strict_logs = flag(host, "matchLogsToLeaves", true);
    let mode = amount(host, "persistentLeavesBehavior", 0., 0., 2.) as u8;
    let mut seen = vec![start];
    let mut queue = VecDeque::from([(start, 1_u8, false)]);
    let mut missing = false;
    while let Some((pos, depth, through_persistent)) = queue.pop_front() {
        for offset in FACES {
            let next = nearby(pos, offset);
            let Some(block) = level.block(host, next) else {
                missing = true;
                continue;
            };
            let name = block.id(host);
            if (is_log(&name) || host.block_in_tag(block, "minecraft:logs") == Some(true))
                && log_match(host, source, &name, strict_logs && !through_persistent) {
                return Some(true);
            }
            if !(is_leaf(&name) || host.block_in_tag(block, "minecraft:leaves") == Some(true))
                || depth >= 6 || seen.contains(&next) { continue; }
            let persistent = level.block_property(host, next, "persistent").as_deref() == Some("true");
            if persistent && mode == 0 { continue; }
            if !(persistent && mode == 2) && !leaf_match(host, source, &name, strict_leaves) {
                continue;
            }
            if seen.len() == MAX_VISITS { return None; }
            seen.push(next);
            queue.push_back((next, depth + 1, through_persistent || (persistent && mode == 2)));
        }
    }
    if missing { None } else { Some(false) }
}

impl Decay {
    fn enqueue(&self, dimension: &str, pos: [i32; 3], retries: u8) {
        let Ok(mut tasks) = self.tasks.lock() else { return; };
        if tasks.len() >= MAX_TASKS || tasks.iter().any(|t| t.dimension == dimension && t.pos == pos) {
            return;
        }
        tasks.push_back(Task { dimension: dimension.to_owned(), pos, due: 0, retries });
    }

    // Committed block changes include natural decay, automated harvest and player breaks.
    // No world writes occur here: update-shape/neighbor notifications may not be done yet.
    pub fn changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        let (Some(old), Some(new)) = (level.state_block(event.old), level.state_block(event.new)) else {
            return;
        };
        let old_log = is_log(old) || level.block_in_tag(event.old.block(), "minecraft:logs") == Some(true);
        let new_log = is_log(new) || level.block_in_tag(event.new.block(), "minecraft:logs") == Some(true);
        let old_leaf = is_leaf(old) || level.block_in_tag(event.old.block(), "minecraft:leaves") == Some(true);
        let new_leaf = is_leaf(new) || level.block_in_tag(event.new.block(), "minecraft:leaves") == Some(true);
        if !((old_log && !new_log) || (old_leaf && !new_leaf)) {
            return;
        }
        let dimension = event.dimension.key().id();
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        for offset in FACES {
            let next = nearby(pos, offset);
            if level.block_state(next).is_some_and(|state| {
                level.state_block(state).is_some_and(is_leaf)
                    || level.block_in_tag(state.block(), "minecraft:leaves") == Some(true)
            }) {
                self.enqueue(dimension, next, 10);
            }
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) { return; }
        let now = self.ticks.fetch_add(1, Ordering::Relaxed) + 1;
        if !on(host, "leaf_decay") || !flag(host, "accelerateLeafDecay", true) {
            self.reset_queue();
            return;
        }
        let minimum = amount(host, "minimumDecayTicks", 5., 0., 1200.) as i32;
        let maximum = amount(host, "maximumDecayTicks", 20., 0., 1200.) as i32;
        let (minimum, maximum) = (minimum.min(maximum), maximum.max(minimum));
        let rate = amount(host, "amountOfLeavesBrokenPerTick", 5., 1., 64.) as usize;
        let checks = self.tasks.lock().map_or(0, |queue| queue.len()).min(MAX_CHECKS * 3);
        let mut processed = 0_usize;
        for _ in 0..checks {
            if processed >= rate { break; }
            let Some(mut task) = self.tasks.lock().ok().and_then(|mut queue| queue.pop_front()) else { break; };
            let Some(level) = host.level(&task.dimension) else { continue; };
            if task.due == 0 {
                // Leaves Be Gone uses a random tick interval rather than a fixed countdown.
                let span = (maximum - minimum).max(1);
                let extra = level.with_random(host, |random| random.next_int_bound(span)).unwrap_or(0);
                task.due = now + (minimum + extra).max(1) as u64;
                self.push(task);
                continue;
            }
            if now < task.due { self.push(task); continue; }
            processed += 1;
            if level.should_tick_blocks_at(host, task.pos) != Some(true) {
                if task.retries > 0 {
                    task.retries -= 1;
                    task.due = now + 20;
                    self.push(task);
                }
                continue;
            }
            let Some(block) = level.block(host, task.pos) else { continue; };
            let name = block.id(host);
            if !(is_leaf(&name) || host.block_in_tag(block, "minecraft:leaves") == Some(true))
                || level.block_property(host, task.pos, "persistent").as_deref() == Some("true") {
                continue;
            }
            match supported(host, level, task.pos, &name) {
                Some(true) => continue,
                None if task.retries > 0 => {
                    task.retries -= 1;
                    task.due = now + 20;
                    self.push(task);
                    continue;
                }
                None => continue,
                Some(false) => {}
            }
            // Prefer vanilla randomTick so leaf loot follows the normal game path.
            // A foreign log might leave vanilla distance under 7: correct that state first.
            if level.block_property(host, task.pos, "distance").as_deref() != Some("7") {
                let _ = host.set_block_property(task.pos, "distance", "7");
            }
            if flag(host, "decayEffects", false) {
                if let Some(state) = level.block_state(host, task.pos) {
                    let _ = level.block_break_effect(host, task.pos, state);
                }
            }
            let _ = level.block_random_tick(host, task.pos);
            if level.block(host, task.pos).is_some_and(|current| current.id(host) == name)
                && level.block_property(host, task.pos, "persistent").as_deref() != Some("true") {
                let _ = level.destroy_block(host, task.pos, true, None, false);
            }
        }
    }

    fn push(&self, task: Task) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if tasks.len() < MAX_TASKS { tasks.push_back(task); }
        }
    }

    fn reset_queue(&self) {
        if let Ok(mut tasks) = self.tasks.lock() { tasks.clear(); }
    }

    pub fn reset(&self) {
        self.reset_queue();
        self.ticks.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_and_species() {
        assert!(is_leaf("minecraft:oak_leaves"));
        assert!(!is_leaf("minecraft:oak_log"));
        assert!(is_log("minecraft:cherry_log"));
        assert_eq!(species("minecraft:pale_oak_leaves"), Some("pale_oak"));
        assert_eq!(species("minecraft:dark_oak_log"), Some("dark_oak"));
    }

    #[test]
    fn neighbor_coordinates() {
        let origin = [0, 64, 0];
        let moved: Vec<_> = FACES.into_iter().map(|offset| nearby(origin, offset)).collect();
        assert_eq!(moved.len(), 6);
        assert!(moved.contains(&[0, 65, 0]));
        assert!(!moved.contains(&[1, 65, 0]));
    }
}
