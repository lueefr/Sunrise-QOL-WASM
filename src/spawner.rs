//! Better Spawner Control: five torches around a normal mob spawner block.
//! A pure, read-only spawn decision: never delete an entity after it has spawned.
use crate::settings::on;
use mod_api::{Host, MobSpawnPositionCheckEvent, SpawnCheck, SpawnReason, Spawner};

fn torch(id: &str) -> bool {
    id == "minecraft:torch" || id == "minecraft:wall_torch"
        || id.ends_with("_torch") || id.ends_with("_wall_torch")
}

fn neighbors(pos: [i32; 3]) -> Option<[[i32; 3]; 5]> {
    let [x, y, z] = pos;
    Some([
        [x, y, z.checked_sub(1)?],
        [x.checked_add(1)?, y, z],
        [x, y, z.checked_add(1)?],
        [x.checked_sub(1)?, y, z],
        [x, y.checked_add(1)?, z],
    ])
}

pub fn check(host: &dyn Host, event: &mut MobSpawnPositionCheckEvent<'_>) {
    if !on(host, "spawner_control") || event.reason != SpawnReason::Spawner {
        return;
    }
    let Some(Spawner::Block { pos }) = event.spawner else { return; };
    let Some(positions) = neighbors([pos.x, pos.y, pos.z]) else { return; };
    // A missing/unloaded block is not counted as a torch, so no false veto occurs.
    if positions.into_iter().all(|location| {
        event.level.block(host, location)
            .is_some_and(|block| torch(&block.id(host)))
    }) {
        *event.result = SpawnCheck::Fail;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn five_expected_sides_without_down() {
        let positions = neighbors([10, 64, 20]).unwrap();
        assert_eq!(positions, [
            [10, 64, 19], [11, 64, 20], [10, 64, 21],
            [9, 64, 20], [10, 65, 20],
        ]);
    }
    #[test]
    fn torch_families() {
        assert!(torch("minecraft:torch"));
        assert!(torch("minecraft:wall_torch"));
        assert!(torch("minecraft:soul_torch"));
        assert!(torch("minecraft:redstone_wall_torch"));
        assert!(!torch("minecraft:torchflower"));
        assert!(!torch("minecraft:lantern"));
    }
}
