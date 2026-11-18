//! Bounded, deterministic scans for WASM: no hash tables or global world pointers.
//! Every host write uses a normal Sunrise server operation instead of native mixins.
use mod_api::{LevelRef, PlayerDestroyedBlockEvent, ServerHost};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::settings::{amount, flag, on};

const MAX_BLOCKS: usize = 1024;
const MAX_VISITS: usize = 16384;
#[derive(Default)]
pub struct Mining {
    running: AtomicBool,
}

struct Guard<'a>(&'a AtomicBool);
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn kind(host: &dyn ServerHost, level: LevelRef, p: [i32; 3]) -> Option<String> {
    level.block(host, p).map(|block| block.id(host))
}

fn log(id: &str) -> bool {
    id.ends_with("_log") || id.ends_with("_wood") || id.ends_with("_stem")
        || id.ends_with("_hyphae") || id == "minecraft:mangrove_roots"
        || id == "minecraft:mutated_mangrove_roots"
}

fn mushroom(id: &str) -> bool {
    id == "minecraft:mushroom_stem"
}

fn family(id: &str) -> &str {
    id.strip_suffix("_log")
        .or_else(|| id.strip_suffix("_wood"))
        .or_else(|| id.strip_suffix("_stem"))
        .or_else(|| id.strip_suffix("_hyphae"))
        .unwrap_or(id)
}

fn same_tree(first: &str, second: &str) -> bool {
    if first == second { return true; }
    if mushroom(first) || mushroom(second) { return false; }
    log(first) && log(second) && family(first) == family(second)
}

fn ore(id: &str) -> bool {
    id.ends_with("_ore") || id == "minecraft:ancient_debris"
}

fn leaf(id: &str) -> bool {
    id.ends_with("_leaves") || matches!(id,
        "minecraft:nether_wart_block" | "minecraft:warped_wart_block"
        | "minecraft:red_mushroom_block" | "minecraft:brown_mushroom_block")
}

fn push_visit(seen: &mut Vec<[i32; 3]>, p: [i32; 3]) -> bool {
    if seen.len() >= MAX_VISITS || seen.contains(&p) { return false; }
    seen.push(p);
    true
}

fn connected(
    host: &dyn ServerHost, level: LevelRef, start: [i32; 3], block: &str,
    is_tree: bool, fuzzy: bool, limit: usize,
) -> Vec<[i32; 3]> {
    let mut queue = VecDeque::from([start]);
    let mut seen = vec![start];
    let mut found = Vec::new();
    while let Some(p) = queue.pop_front() {
        if found.len() >= limit || seen.len() >= MAX_VISITS { break; }
        let centers = if is_tree { 2 } else { 1 };
        for center in 0..centers {
            for dx in -1_i32..=1 {
                for dy in -1_i32..=1 {
                    for dz in -1_i32..=1 {
                        if dx == 0 && dy == 0 && dz == 0 && center == 0 { continue; }
                        if !is_tree && !fuzzy && dx.abs() + dy.abs() + dz.abs() != 1 { continue; }
                        let next = [p[0] + dx, p[1] + dy + 2 * center, p[2] + dz];
                        if !push_visit(&mut seen, next) { continue; }
                        let Some(next_id) = kind(host, level, next) else { continue; };
                        let matching = if is_tree {
                            same_tree(block, &next_id)
                        } else {
                            block == next_id
                        };
                        if matching && found.len() < limit {
                            queue.push_back(next);
                            if next != start { found.push(next); }
                        }
                    }
                }
            }
        }
    }
    found
}

fn has_canopy(host: &dyn ServerHost, level: LevelRef, logs: &[[i32; 3]]) -> bool {
    // A natural-looking canopy is required before any additional log is broken.
    for center in logs.iter().rev().take(64) {
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..=3 {
                    let p = [center[0] + dx, center[1] + dy, center[2] + dz];
                    if kind(host, level, p).as_deref().is_some_and(leaf)
                        && level.block_property(host, p, "persistent").as_deref() != Some("true") {
                        return true;
                    }
                }
            }
        }
    }
    false
}


impl Mining {
    pub fn block(&self, host: &mut dyn ServerHost, event: &mut PlayerDestroyedBlockEvent) {
        if self.running.swap(true, Ordering::AcqRel) { return; }
        let _guard = Guard(&self.running);
        let source = event.state.block().id(host);
        let tree = on(host, "tree_harvester") && log(&source);
        let vein = on(host, "vein_miner") && ore(&source);
        if !tree && !vein { return; }
        let Some(player) = host.players().acting_player() else { return; };
        if player.is_creative(host) != Some(false) { return; }
        let sneaking = player.is_secondary_use_active(host) == Some(true);
        if tree && !flag(host, "treeHarvestIgnoresCrouching", false) {
            let without = flag(host, "treeHarvestWithoutCrouching", false);
            if sneaking == without { return; }
        }
        if vein && sneaking == flag(host, "oreHarvestWithoutSneak", false) { return; }
        let Some(level) = player.level(host) else { return; };
        let Some(slot) = player.selected_slot(host) else { return; };
        let Some(tool) = player.inventory_slots(host).unwrap_or_default().into_iter()
            .find(|(index, _)| *index as usize == slot)
            .map(|(_, stack)| stack.item().id(host)) else { return; };
        if tree {
            if flag(host, "mustHoldAxeForTreeHarvest", true) && !tool.ends_with("_axe") {
                return;
            }
            if !flag(host, "enableNetherTrees", true)
                && (source.ends_with("_stem") || source.ends_with("_hyphae")) {
                return;
            }
            if !flag(host, "enableHugeMushrooms", true) && mushroom(&source) {
                return;
            }
        }
        if vein && !tool.ends_with("_pickaxe") { return; }
        let limit = if tree {
            amount(host, "maxAmountOfLogsBrokenPerHarvest", 256., 1., MAX_BLOCKS as f64)
        } else {
            amount(host, "maxOresPerVein", 256., 1., MAX_BLOCKS as f64)
        } as usize;
        let fuzzy = flag(host, "enableFuzzyOreSearch", true);
        let mut logs = connected(host, level, event.pos, &source, tree, fuzzy, limit);
        if tree {
            logs.push(event.pos);
            if flag(host, "ignorePlayerMadeTrees", true) && !has_canopy(host, level, &logs) {
                return;
            }
            logs.pop();
        }
        for p in logs {
            let Some(block) = kind(host, level, p) else { continue; };
            if !(if tree { same_tree(&source, &block) } else { block == source }) { continue; }
            // Preserve server-side vanilla drops, events, XP and tool durability.
            let _ = player.destroy_block(host, p, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_families() {
        assert!(same_tree("minecraft:oak_log", "minecraft:oak_wood"));
        assert!(!same_tree("minecraft:oak_log", "minecraft:spruce_log"));
        assert!(ore("minecraft:deepslate_diamond_ore"));
        assert!(ore("minecraft:ancient_debris"));
        assert!(!ore("minecraft:stone"));
    }

    #[test]
    fn bounded_deduplication() {
        let mut seen = vec![[1, 2, 3]];
        assert!(!push_visit(&mut seen, [1, 2, 3]));
        assert!(push_visit(&mut seen, [2, 2, 3]));
        assert_eq!(seen.len(), 2);
    }
}
