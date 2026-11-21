//! Keep My Soil Tilled: replace dirt beneath a harvested melon or pumpkin
//! only when neighbouring farmland shows that the fruit grew beside a stem.
//! The original only handles player-harvested stem fruit, not arbitrary breaks.
use crate::settings::on;
use mod_api::{BlockUpdateFlags, PlayerDestroyedBlockEvent, ServerHost};

fn fruit(id: &str) -> bool {
    matches!(id, "minecraft:melon" | "minecraft:pumpkin")
}

fn neighbours(pos: [i32; 3]) -> Option<Vec<[i32; 3]>> {
    let y = pos[1].checked_sub(1)?;
    let mut result = Vec::with_capacity(9);
    for dx in -1_i32..=1 {
        for dz in -1_i32..=1 {
            result.push([
                pos[0].checked_add(dx)?,
                y,
                pos[2].checked_add(dz)?,
            ]);
        }
    }
    Some(result)
}

pub fn harvested(host: &mut dyn ServerHost, event: &PlayerDestroyedBlockEvent) {
    if !on(host, "keep_soil_tilled") || !fruit(&event.state.block().id(host)) {
        return;
    }
    let Some(player) = host.players().acting_player() else { return; };
    let Some(level) = player.level(host) else { return; };
    if !level.in_place(host) { return; }
    let Some(positions) = neighbours(event.pos) else { return; };
    // Upstream searches the 3x3 layer below the harvested pumpkin or melon.
    let near_farmland = positions.iter().any(|&pos| {
        level.block(host, pos).is_some_and(|block| block.id(host) == "minecraft:farmland")
    });
    if !near_farmland { return; }
    let Some(y) = event.pos[1].checked_sub(1) else { return; };
    let below = [event.pos[0], y, event.pos[2]];
    let Some(existing) = level.block_state(host, below) else { return; };
    if host.block_in_tag(existing.block(), "minecraft:dirt") != Some(true) { return; }
    let Some(farmland) = host.block_state_id("minecraft:farmland") else { return; };
    let _ = level.set_block(host, below, farmland,
        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_stem_fruit() {
        assert!(fruit("minecraft:melon"));
        assert!(fruit("minecraft:pumpkin"));
        assert!(!fruit("minecraft:carved_pumpkin"));
        assert!(!fruit("minecraft:wheat"));
    }

    #[test]
    fn searches_the_original_lower_neighbourhood() {
        let near = neighbours([10, 64, 20]).unwrap();
        assert_eq!(near.len(), 9);
        assert!(near.contains(&[9, 63, 19]));
        assert!(near.contains(&[10, 63, 20]));
        assert!(near.contains(&[11, 63, 21]));
    }
}
