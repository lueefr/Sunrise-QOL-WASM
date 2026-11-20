//! Zombie Proof Doors: prevent the zombie's existing BreakDoor goal
//! from running. Upstream clears Zombie.canBreakDoors when a mob joins.
use crate::settings::on;
use mod_api::{EntityJoinLevelEvent, ServerHost};

fn zombie(kind: &str) -> bool {
    matches!(kind,
        "minecraft:zombie" | "minecraft:husk" | "minecraft:drowned"
        | "minecraft:zombie_villager")
}

pub fn joined(host: &mut dyn ServerHost, event: &EntityJoinLevelEvent<'_>) {
    if !on(host, "zombie_proof_doors") || !zombie(event.kind) {
        return;
    }
    let Some(entity) = host.entity(event.entity) else { return; };
    let Some(mob) = host.mob(entity) else { return; };
    // In Sunrise the door-breaking behaviour lives in a BreakDoor goal;
    // deleting that goal stops door targeting before it can reach the block.
    let _ = mob.remove_goal(host, "BreakDoor");
    // The source does not disable ordinary door-opening navigation.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_zombie_subclasses() {
        for name in ["zombie", "husk", "drowned", "zombie_villager"] {
            assert!(zombie(&format!("minecraft:{name}")));
        }
        assert!(!zombie("minecraft:vindicator"));
        assert!(!zombie("minecraft:creeper"));
        assert!(!zombie("minecraft:zombified_piglin"));
    }
}
