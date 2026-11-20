//! Bottled Air: restore oxygen from glass bottles under water.
//! The ordinary vanilla bottle action is not canceled, as in the source mod.
use crate::settings::{amount, flag, on};
use mod_api::{
    ComponentId, Flow, Hand, HostSide, InteractionResult, ItemResource, ItemStack, Identifier,
    JavaString, RightClickItemEvent, ServerHost, Tag,
};

fn fire_item(id: &str) -> bool {
    matches!(id, "minecraft:campfire" | "minecraft:soul_campfire"
        | "minecraft:fire_charge" | "minecraft:lava_bucket" | "minecraft:magma_block")
}

fn water_potion(host: &dyn ServerHost, stack: &ItemStack) -> bool {
    if stack.item().id(host) != "minecraft:potion" { return false; }
    let Some(id) = ComponentId::new("minecraft:potion_contents") else { return false; };
    let Some(Tag::Compound(data)) = ItemResource::of(stack).value(host, &id) else {
        return false;
    };
    matches!(data.get(&JavaString::from("potion")),
        Some(Tag::String(name)) if name.to_string().ok().as_deref() == Some("minecraft:water"))
}

fn other_slot(host: &dyn ServerHost, player: mod_api::PlayerRef, hand: Hand) -> Option<u32> {
    match hand {
        Hand::Main => Some(40),
        Hand::Off => player.selected_slot(host).and_then(|slot| u32::try_from(slot).ok()),
    }
}

pub fn use_bottle(side: HostSide<'_>, event: &mut RightClickItemEvent<'_>) -> Flow {
    let HostSide::Server(host) = side else { return Flow::Continue; };
    if !on(host, "bottled_air") { return Flow::Continue; }
    let Some(player) = host.player(event.click.player) else { return Flow::Continue; };
    let entity = player.entity();
    if entity.is_in_water(host) != Some(true) { return Flow::Continue; }

    let held = event.click.stack;
    if held.item().id(host) != "minecraft:glass_bottle" {
        if flag(host, "disableWaterConsumptionUnderwater", true)
            && water_potion(host, held) && entity.is_under_water(host) == Some(true) {
            *event.cancellation_result = InteractionResult::Fail;
            return Flow::Cancel;
        }
        return Flow::Continue;
    }

    let (Some(current), Some(max)) = (entity.air_supply(host), entity.max_air_supply(host))
        else { return Flow::Continue; };
    if current >= max { return Flow::Continue; }
    let restored = amount(host, "bottled_air_amount", 150., 0., 1000.) as i32;
    if restored <= 0 || !entity.set_air_supply(host, current.saturating_add(restored).min(max)) {
        return Flow::Continue;
    }

    if !flag(host, "holdFireTypeItemInOffhandToPreventWaterBottleCreation", true) {
        return Flow::Continue;
    }
    let Some(other) = other_slot(host, player, event.click.hand) else {
        return Flow::Continue;
    };
    let Some(slots) = player.inventory_slots(host) else { return Flow::Continue; };
    if !slots.iter().any(|(slot, stack)| *slot == other && fire_item(&stack.item().id(host))) {
        return Flow::Continue;
    }

    // Upstream searches inventory slots 35 down through 1, never slot 0.
    let target = slots.into_iter().filter(|(slot, stack)| {
        (1..=35).contains(slot) && water_potion(host, stack)
    }).max_by_key(|(slot, _)| *slot);
    let Some((slot, mut water)) = target else { return Flow::Continue; };
    let count = water.count().saturating_sub(1);
    let next = if count == 0 { None }
        else if water.set_count(count).is_ok() { Some(water) }
        else { return Flow::Continue; };
    if player.set_inventory_slot(host, slot, next).is_err() { return Flow::Continue; }

    // Collective uses a separate global RNG; the public Sunrise server
    // exposes its level RNG instead, so the random sequence is not identical.
    let chance = amount(host, "chanceGlassBottleBreaksWithFireTypeInOffhand", 0.5, 0., 1.);
    let Some(level) = player.level(host) else { return Flow::Continue; };
    let survives = level.with_random(host, |random| random.next_double() > chance)
        .unwrap_or(false);
    if survives {
        if let Some(item) = Identifier::parse("minecraft:glass_bottle")
            .ok().and_then(|id| host.item_key(&id)) {
            if let Some(stack) = ItemStack::new(item, 1) {
                let _ = player.give_stack(host, stack);
            }
        }
    }
    Flow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_items_match_original() {
        for id in ["minecraft:campfire", "minecraft:soul_campfire", "minecraft:fire_charge",
                   "minecraft:lava_bucket", "minecraft:magma_block"] {
            assert!(fire_item(id));
        }
        assert!(!fire_item("minecraft:torch"));
    }

    #[test]
    fn oxygen_is_capped() {
        assert_eq!(210_i32.saturating_add(150).min(300), 300);
        assert_eq!((-20_i32).saturating_add(150).min(300), 130);
    }
}
