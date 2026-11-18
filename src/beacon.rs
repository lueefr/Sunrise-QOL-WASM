//! Beacon Placement: build a mineral base using blocks from the clicked player's hand.
//! The public WASM placement operation does not enforce all vanilla use-on checks,
//! so this implementation requires empty target positions and build permissions.
use crate::settings::{flag, on};
use mod_api::{
    BlockFace, BlockUpdateFlags, Flow, Hand, PlayerDestroyedBlockEvent,
    RightClickBlockEvent, ServerHost,
};
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_BASE: usize = 164;

fn mineral(name: &str) -> bool {
    matches!(name,
        "minecraft:iron_block" | "minecraft:gold_block" | "minecraft:diamond_block"
        | "minecraft:emerald_block" | "minecraft:netherite_block")
}

fn empty(name: &str) -> bool {
    matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

fn locations(pos: [i32; 3]) -> Vec<[i32; 3]> {
    let mut result = Vec::with_capacity(MAX_BASE);
    for layer in 1_i32..=4 {
        let Some(y) = pos[1].checked_sub(layer) else { break; };
        for dx in -layer..=layer {
            for dz in -layer..=layer {
                let (Some(x), Some(z)) = (pos[0].checked_add(dx), pos[2].checked_add(dz)) else {
                    continue;
                };
                result.push([x, y, z]);
            }
        }
    }
    result
}

#[derive(Default)]
pub struct Beacon {
    breaking: AtomicBool,
}

struct BreakGuard<'a>(&'a AtomicBool);
impl Drop for BreakGuard<'_> {
    fn drop(&mut self) { self.0.store(false, Ordering::Release); }
}

impl Beacon {
    pub fn click(&self, host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
        if !on(host, "beacon_placement") { return Flow::Continue; }
        let Some(player) = host.players().acting_player() else { return Flow::Continue; };
        if player.may_build(host) != Some(true) || player.is_spectator(host) != Some(false) {
            return Flow::Continue;
        }
        let Some(level) = player.level(host) else { return Flow::Continue; };
        if !level.in_place(host)
            || level.block(host, event.click.pos).is_none_or(|block| block.id(host) != "minecraft:beacon") {
            return Flow::Continue;
        }
        let Some(stack) = event.click.stack.cloned() else { return Flow::Continue; };
        let material = stack.item().id(host);
        if !mineral(&material) || stack.count() == 0 { return Flow::Continue; }
        if player.may_use_item_at(host, event.click.pos, BlockFace::Up, &stack) != Some(true) {
            return Flow::Continue;
        }
        let Some(state) = host.block_state_id(&material) else { return Flow::Continue; };
        let slot = match event.click.hand {
            Hand::Main => player.selected_slot(host).and_then(|value| u32::try_from(value).ok()),
            Hand::Off => Some(40),
        };
        let Some(slot) = slot else { return Flow::Continue; };
        let creative = player.is_creative(host) == Some(true);
        let many = player.is_secondary_use_active(host) == Some(true);
        let limit = if many { MAX_BASE } else { 1 };
        let mut placed = 0;
        for pos in locations(event.click.pos) {
            if placed >= limit { break; }
            let Some(previous) = level.block_state(host, pos) else { continue; };
            let Some(block) = level.block(host, pos) else { continue; };
            if !empty(&block.id(host)) { continue; }
            // Confirm the same block is still held before spending it.
            let Some(mut held) = player.inventory_slots(host).and_then(|list| {
                list.into_iter().find(|(index, _)| *index == slot).map(|(_, stack)| stack)
            }) else { break; };
            if held.item().id(host) != material || held.count() == 0 { break; }
            if player.place_block(host, level, pos, state, true).is_err() { continue; }
            if !creative {
                let count = held.count() - 1;
                let next = if count == 0 { None } else if held.set_count(count) { Some(held) } else {
                    let _ = level.set_block(host, pos, previous,
                        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS);
                    break;
                };
                if player.set_inventory_slot(host, slot, next).is_err() {
                    let _ = level.set_block(host, pos, previous,
                        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS);
                    break;
                }
            }
            placed += 1;
        }
        if placed > 0 { Flow::Cancel } else { Flow::Continue }
    }

    pub fn broken(&self, host: &mut dyn ServerHost, event: &PlayerDestroyedBlockEvent) {
        if event.state.block().id(host) != "minecraft:beacon"
            || !on(host, "beacon_placement")
            || !flag(host, "beacon_break_base", true) {
            return;
        }
        if self.breaking.swap(true, Ordering::AcqRel) { return; }
        let _guard = BreakGuard(&self.breaking);
        let Some(player) = host.players().acting_player() else { return; };
        let Some(level) = player.level(host) else { return; };
        if !level.in_place(host) { return; }
        for pos in locations(event.pos) {
            if let Some(block) = level.block(host, pos) {
                if mineral(&block.id(host)) {
                    // Keep the normal break and drops; an unloaded or protected block is untouched.
                    let _ = player.destroy_block(host, pos, false);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn base_is_bounded_and_ordered_by_layer() {
        let positions = locations([0, 64, 0]);
        assert_eq!(positions.len(), MAX_BASE);
        assert_eq!(positions[0], [-1, 63, -1]);
        assert_eq!(positions[8], [1, 63, 1]);
        assert_eq!(positions[9], [-2, 62, -2]);
        assert_eq!(positions.last(), Some(&[4, 60, 4]));
    }
    #[test]
    fn only_valid_minerals_and_empty_targets() {
        assert!(mineral("minecraft:netherite_block"));
        assert!(!mineral("minecraft:iron_ore"));
        assert!(!mineral("modded:random_block"));
        assert!(empty("minecraft:air"));
        assert!(!empty("minecraft:bedrock"));
    }
}
