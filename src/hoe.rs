//! Hoe Tweaks: server-authoritative square tilling and undo-tilling using the
//! original tier radii and crouch rules. Pumpkin/melon mining speed requires a
//! break-speed override not currently exposed by the public WASM callbacks.
use crate::settings::{amount, flag, on};
use mod_api::{
    BlockFace, BlockUpdateFlags, EquipmentSlot, Flow, Hand, Host, HostSide,
    InteractionResult, PlayerBreakSpeedEvent, PlayerRef, RightClickBlockEvent, ServerHost,
};

fn melon_or_pumpkin(id: &str) -> bool {
    matches!(id, "minecraft:pumpkin" | "minecraft:melon")
}

fn apply_speed(host: &dyn Host, player: PlayerRef, event: &mut PlayerBreakSpeedEvent) {
    if !on(host, "hoe_tweaks") || player.entity().id() != event.player {
        return;
    }
    let Some(level) = player.entity().level(host) else { return; };
    if !level.block(host, event.pos).is_some_and(|block| {
        melon_or_pumpkin(&block.id(host))
    }) { return; }
    let Some(selected) = player.selected_slot(host) else { return; };
    let has_hoe = player.inventory_slots(host).is_some_and(|slots| slots.into_iter().any(
        |(slot, stack)| usize::try_from(slot).ok() == Some(selected)
            && hoe(&stack.item().id(host))
    ));
    if has_hoe {
        // The original replaces the break speed with its configured value,
        // rather than multiplying the vanilla result.
        event.new_speed = amount(host, "cropBlockBreakSpeedModifier", 8., 0., 20.) as f32;
    }
}

pub fn speed(side: HostSide<'_>, event: &mut PlayerBreakSpeedEvent) -> Flow {
    match side {
        HostSide::Client(host) => {
            if let Some(player) = host.local_player() {
                apply_speed(host, player, event);
            }
        }
        HostSide::Server(host) => {
            if let Some(player) = host.players().acting_player() {
                apply_speed(host, player, event);
            }
        }
    }
    Flow::Continue
}

fn hoe(id: &str) -> bool {
    id.ends_with("_hoe")
}

fn radius(host: &dyn ServerHost, id: &str) -> i32 {
    let material = id.rsplit('/').next().unwrap_or(id)
        .rsplit(':').next().unwrap_or(id);
    let (key, default) = if material.starts_with("netherite_") {
        ("netheriteTierHoeRange", 4.)
    } else if material.starts_with("diamond_") {
        ("diamondTierHoeRange", 3.)
    } else if material.starts_with("iron_") || material.starts_with("steel_") {
        ("ironTierHoeRange", 2.)
    } else if material.starts_with("gold_") {
        ("goldTierHoeRange", 2.)
    } else if material.starts_with("stone_") {
        ("stoneTierHoeRange", 1.)
    } else {
        ("woodenTierHoeRange", 0.)
    };
    amount(host, key, default, 0., 32.) as i32
}

fn dirt(host: &dyn ServerHost, block: mod_api::BlockKey) -> bool {
    let id = block.id(host);
    matches!(id.as_str(),
        "minecraft:dirt" | "minecraft:grass_block" | "minecraft:coarse_dirt"
        | "minecraft:rooted_dirt" | "minecraft:podzol")
        || host.block_in_tag(block, "minecraft:dirt") == Some(true)
}

fn air(id: &str) -> bool {
    matches!(id, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

fn bush(host: &dyn ServerHost, block: mod_api::BlockKey) -> bool {
    let id = block.id(host);
    matches!(id.as_str(),
        "minecraft:short_grass" | "minecraft:grass" | "minecraft:fern"
        | "minecraft:large_fern" | "minecraft:tall_grass" | "minecraft:dead_bush"
        | "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes"
        | "minecraft:beetroots" | "minecraft:nether_wart"
        | "minecraft:sweet_berry_bush" | "minecraft:sapling"
        | "minecraft:torchflower_crop" | "minecraft:pitcher_crop")
        || host.block_in_tag(block, "minecraft:flowers") == Some(true)
        || host.block_in_tag(block, "minecraft:saplings") == Some(true)
}

fn square(center: [i32; 3], radius: i32) -> Vec<[i32; 3]> {
    let side = radius.saturating_mul(2).saturating_add(1) as usize;
    let mut positions = Vec::with_capacity(side.saturating_mul(side));
    for x in -radius..=radius {
        for z in -radius..=radius {
            if let (Some(px), Some(pz)) =
                (center[0].checked_add(x), center[2].checked_add(z)) {
                positions.push([px, center[1], pz]);
            }
        }
    }
    positions
}

fn alternate_hand_free(host: &dyn ServerHost, player: mod_api::PlayerRef, hand: Hand) -> bool {
    let slot = match hand {
        Hand::Main => Some(40),
        Hand::Off => player.selected_slot(host).and_then(|value| u32::try_from(value).ok()),
    };
    let Some(slot) = slot else { return false; };
    player.inventory_slots(host).is_some_and(|slots| {
        !slots.into_iter().any(|(index, stack)| index == slot && stack.count() > 0)
    })
}

pub fn click(host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "hoe_tweaks") { return Flow::Continue; }
    let Some(held) = event.click.stack else { return Flow::Continue; };
    let item = held.item().id(host);
    if !hoe(&item) || held.count() == 0 { return Flow::Continue; }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if player.may_build(host) != Some(true)
        || player.is_spectator(host) != Some(false) {
        return Flow::Continue;
    }
    // As in Hoe Tweaks, holding an item in the other hand bypasses the
    // custom action, so seed placement remains available.
    if flag(host, "onlyUntillWithOtherHandEmpty", true)
        && !alternate_hand_free(host, player, event.click.hand) {
        return Flow::Continue;
    }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }
    let center = event.click.pos;
    let Some(block) = level.block(host, center) else { return Flow::Continue; };
    let untill = block.id(host) == "minecraft:farmland";
    if !untill && !dirt(host, block) { return Flow::Continue; }

    let crouching = player.is_secondary_use_active(host) == Some(true);
    let crouch_required = flag(host, "mustCrouchToHaveBiggerHoeRange", true);
    if !untill && !crouching && crouch_required {
        // Ordinary vanilla hoe tilling handles the single dirt block.
        return Flow::Continue;
    }
    let radius = if !crouching && crouch_required { 0 }
        else { radius(host, &item) };
    let Some(target) = host.block_state_id(if untill {
        "minecraft:dirt"
    } else {
        "minecraft:farmland"
    }) else { return Flow::Continue; };

    let positions = square(center, radius);
    let mut changed: i32 = 0;
    for pos in positions {
        let Some(current) = level.block_state(host, pos) else { continue; };
        let matches = if untill {
            current.block().id(host) == "minecraft:farmland"
        } else {
            dirt(host, current.block())
        };
        if !matches { continue; }
        if player.may_use_item_at(host, pos, BlockFace::Up, held) != Some(true) {
            continue;
        }
        let Some(y) = pos[1].checked_add(1) else { continue; };
        let above = [pos[0], y, pos[2]];
        let Some(plant) = level.block(host, above) else { continue; };
        let blocked = !air(&plant.id(host));
        if blocked && !bush(host, plant) { continue; }
        if blocked {
            // BushBlock is removed with drops, without spending the held
            // hoe's durability as a normal player-block break would.
            if level.destroy_block(host, above, true, None, false)
                .ok().flatten().is_none() {
                continue;
            }
        }
        if level.set_block(host, pos, target,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS) == Ok(true) {
            changed += 1;
        }
    }
    if changed == 0 { return Flow::Continue; }
    if player.is_creative(host) != Some(true) {
        if let Some(living) = host.living(player.entity()) {
            let slot = match event.click.hand {
                Hand::Main => EquipmentSlot::MainHand,
                Hand::Off => EquipmentSlot::OffHand,
            };
            let _ = living.hurt_and_break(host, slot, changed);
        }
    }
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_matches_original_inclusive_range() {
        assert_eq!(square([0, 64, 0], 0).len(), 1);
        assert_eq!(square([0, 64, 0], 1).len(), 9);
        assert_eq!(square([0, 64, 0], 2).len(), 25);
        assert_eq!(square([0, 64, 0], 4).len(), 81);
        assert_eq!(square([0, 64, 0], 32).len(), 4225);
    }

    #[test]
    fn hoe_ids_and_air() {
        assert!(hoe("minecraft:netherite_hoe"));
        assert!(hoe("modid:steel_hoe"));
        assert!(!hoe("minecraft:diamond_axe"));
        assert!(melon_or_pumpkin("minecraft:melon"));
        assert!(melon_or_pumpkin("minecraft:pumpkin"));
        assert!(!melon_or_pumpkin("minecraft:wheat"));
        assert!(air("minecraft:cave_air"));
    }
}
