//! Inventory Totem attempts the same death protection as a held vanilla totem.
//! Only the actual vanilla totem is handled here, never a guessed modded item.
use crate::settings::on;
use mod_api::{ActiveEffect, Flow, LivingDeathEvent, ServerHost};

fn bypasses(source: &str) -> bool {
    matches!(source, "minecraft:generic_kill" | "minecraft:out_of_world")
}

pub fn on_death(host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
    if event.kind != "minecraft:player" || !on(host, "inventory_totem")
        || bypasses(event.source.damage_type) { return Flow::Continue; }
    let Some(player) = host.players().server_players().into_iter().find(|p| {
        i32::try_from(p.entity().id()).ok() == Some(event.entity)
    }) else { return Flow::Continue; };
    let selected = player.selected_slot(host);
    let Some((slot, mut stack)) = player.inventory_slots(host)
        .and_then(|slots| slots.into_iter().find(|(slot, stack)| {
            *slot != 40 && Some(*slot) != selected
                && stack.item().id(host) == "minecraft:totem_of_undying"
        })) else { return Flow::Continue; };
    // The full effect bundle and the inventory write use server-authoritative APIs.
    let Some(entity) = host.entity(event.entity) else { return Flow::Continue; };
    let Some(living) = host.living(entity) else { return Flow::Continue; };
    let rest = if stack.count() == 1 { None } else {
        let next = stack.count() - 1;
        if stack.set_count(next).is_err() { return Flow::Continue; }
        Some(stack)
    };
    // Never consume an inventory totem if the authoritative health write fails.
    let original = player.inventory_slots(host).and_then(|slots| {
        slots.into_iter().find(|(index, _)| *index == slot).map(|(_, stack)| stack)
    });
    if player.set_inventory_slot(host, slot, rest).is_err() { return Flow::Continue; }
    if !living.set_health(host, 1.) {
        if let Some(stack) = original { let _ = player.set_inventory_slot(host, slot, Some(stack)); }
        return Flow::Continue;
    }
    let _ = living.remove_all_effects(host);
    for (name, duration, amplifier) in [
        ("minecraft:regeneration", 900, 1),
        ("minecraft:absorption", 100, 1),
        ("minecraft:fire_resistance", 800, 0),
    ] {
        if let Some(id) = host.effect_key(name) {
            let _ = living.add_effect(host, &ActiveEffect::new(id, duration, amplifier, false, true, true));
        }
    }
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vanilla_bypass_sources_do_not_trigger_inventory_totem() {
        assert!(bypasses("minecraft:generic_kill"));
        assert!(bypasses("minecraft:out_of_world"));
        assert!(!bypasses("minecraft:fall"));
    }
}
