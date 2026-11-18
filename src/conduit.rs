//! Conduit Placement: construct the canonical 42-block conduit frame from hand items.
//! Only empty and water-filled positions are used; unrelated structures remain untouched.
use crate::settings::{flag, on};
use mod_api::{
    BlockFace, BlockUpdateFlags, Flow, Hand, PlayerDestroyedBlockEvent,
    RightClickBlockEvent, ServerHost,
};
use std::sync::atomic::{AtomicBool, Ordering};

const FRAME_SIZE: usize = 42;

fn frame_block(id: &str) -> bool {
    matches!(id, "minecraft:prismarine" | "minecraft:dark_prismarine"
        | "minecraft:prismarine_bricks" | "minecraft:sea_lantern")
}

fn replaceable(id: &str) -> bool {
    matches!(id, "minecraft:water" | "minecraft:air" | "minecraft:cave_air")
}

fn frame(center: [i32; 3]) -> Vec<[i32; 3]> {
    let mut result = Vec::with_capacity(FRAME_SIZE);
    for plane in 0..3 {
        for a in -2_i32..=2 {
            for b in -2_i32..=2 {
                if a.abs() != 2 && b.abs() != 2 { continue; }
                let offset = match plane {
                    0 => [a, b, 0],
                    1 => [a, 0, b],
                    _ => [0, a, b],
                };
                let (Some(x), Some(y), Some(z)) =
                    (center[0].checked_add(offset[0]),
                     center[1].checked_add(offset[1]),
                     center[2].checked_add(offset[2])) else { continue; };
                let pos = [x, y, z];
                if !result.contains(&pos) { result.push(pos); }
            }
        }
    }
    result
}

#[derive(Default)]
pub struct Conduit {
    breaking: AtomicBool,
}

struct BreakGuard<'a>(&'a AtomicBool);
impl Drop for BreakGuard<'_> {
    fn drop(&mut self) { self.0.store(false, Ordering::Release); }
}

impl Conduit {
    pub fn click(&self, host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
        if !on(host, "conduit_placement") { return Flow::Continue; }
        let Some(player) = host.players().acting_player() else { return Flow::Continue; };
        if player.may_build(host) != Some(true) || player.is_spectator(host) != Some(false) {
            return Flow::Continue;
        }
        let Some(level) = player.level(host) else { return Flow::Continue; };
        if !level.in_place(host)
            || level.block(host, event.click.pos).is_none_or(|block| block.id(host) != "minecraft:conduit") {
            return Flow::Continue;
        }
        let Some(stack) = event.click.stack.cloned() else { return Flow::Continue; };
        let material = stack.item().id(host);
        if !frame_block(&material) || stack.count() == 0 { return Flow::Continue; }
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
        let multi = player.is_secondary_use_active(host) == Some(true);
        let limit = if multi { FRAME_SIZE } else { 1 };
        let mut placed = 0;
        for pos in frame(event.click.pos) {
            if placed >= limit { break; }
            let Some(previous) = level.block_state(host, pos) else { continue; };
            let Some(block) = level.block(host, pos) else { continue; };
            if !replaceable(&block.id(host)) { continue; }
            let Some(mut held) = player.inventory_slots(host).and_then(|list| {
                list.into_iter().find(|(index, _)| *index == slot).map(|(_, stack)| stack)
            }) else { break; };
            if held.item().id(host) != material || held.count() == 0 { break; }
            if player.place_block(host, level, pos, state, true).is_err() { continue; }
            if !creative {
                let count = held.count() - 1;
                let next = if count == 0 { None } else if held.set_count(count).is_ok() {
                    Some(held)
                } else {
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
        if event.state.block().id(host) != "minecraft:conduit"
            || !on(host, "conduit_placement")
            || !flag(host, "conduit_break_frame", true) {
            return;
        }
        if self.breaking.swap(true, Ordering::AcqRel) { return; }
        let _guard = BreakGuard(&self.breaking);
        let Some(player) = host.players().acting_player() else { return; };
        let Some(level) = player.level(host) else { return; };
        if !level.in_place(host) { return; }
        for pos in frame(event.pos) {
            let Some(block) = level.block(host, pos) else { continue; };
            if frame_block(&block.id(host)) {
                let _ = player.destroy_block(host, pos, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conduit_frame_has_exactly_42_unique_positions() {
        let positions = frame([0, 64, 0]);
        assert_eq!(positions.len(), FRAME_SIZE);
        assert!(positions.contains(&[2, 64, 0]));
        assert!(positions.contains(&[0, 66, 2]));
        assert!(positions.contains(&[-2, 62, 0]));
        assert!(!positions.contains(&[0, 64, 0]));
    }

    #[test]
    fn only_conduit_minerals_can_be_placed() {
        assert!(frame_block("minecraft:prismarine_bricks"));
        assert!(frame_block("minecraft:sea_lantern"));
        assert!(!frame_block("minecraft:stone"));
        assert!(replaceable("minecraft:water"));
        assert!(!replaceable("minecraft:chest"));
    }
}
