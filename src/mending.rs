//! Inventory Mending is a fallback for the original XP-orb repair mechanic.
//! The public XP-change callback is later than the vanilla enchanted-item
//! selection, so skip the fallback when equipped Mending gear still needs
//! repair. At most one inventory item is repaired per XP event.
use crate::settings::{flag, on};
use mod_api::{ComponentId, Flow, ItemResource, JavaString, PlayerXpChangeEvent, ServerHost, Tag};

fn has_mending(host: &dyn ServerHost, stack: &mod_api::ItemStack) -> bool {
    let Some(id) = ComponentId::new("minecraft:enchantments") else { return false; };
    let Some(Tag::Compound(levels)) = ItemResource::of(stack).value(host, &id) else { return false; };
    levels.get(&JavaString::from("minecraft:mending")).is_some()
}

fn equipped(slot: u32, selected: usize) -> bool {
    slot >= 36 || usize::try_from(slot).ok() == Some(selected)
}

fn eligible(slot: u32, selected: usize, hotbar_only: bool) -> bool {
    slot <= 35 && (!hotbar_only || slot <= 8) && !equipped(slot, selected)
}

fn repair(stack: &mut mod_api::ItemStack, available: i32) -> i32 {
    let cost = stack.damage().div_ceil(2).min(available.max(0) as u32);
    if cost == 0 { return 0; }
    stack.set_damage(stack.damage().saturating_sub(cost.saturating_mul(2)));
    cost as i32
}

pub fn on_xp(host: &mut dyn ServerHost, event: &mut PlayerXpChangeEvent<'_>) -> Flow {
    if event.amount <= 0 || !on(host, "inventory_mending") { return Flow::Continue; }
    let Some(player) = host.player(event.player) else { return Flow::Continue; };
    let Some(slots) = player.inventory_slots(host) else { return Flow::Continue; };
    let Some(selected) = player.selected_slot(host) else { return Flow::Continue; };

    // Upstream only considers inventory slots if Minecraft found no
    // applicable equipped stack. This guard prevents the earlier version
    // from repairing a second item after an equipped item was selected.
    if slots.iter().any(|(slot, stack)| {
        equipped(*slot, selected) && stack.damage() > 0 && has_mending(host, stack)
    }) {
        return Flow::Continue;
    }

    let hotbar_only = flag(host, "mending_hotbar_only", false);
    let candidates: Vec<_> = slots.into_iter().filter(|(slot, stack)| {
        eligible(*slot, selected, hotbar_only)
            && stack.damage() > 0 && has_mending(host, stack)
    }).collect();
    if candidates.is_empty() { return Flow::Continue; }

    // The source uses the player's RNG while processing the experience orb.
    // The WASM API exposes a level RNG, not that per-player orb RNG.
    let Some(level) = player.level(host) else { return Flow::Continue; };
    let count = candidates.len() as i32;
    let Some(pick) = level.with_random(host, |random| random.next_int_bound(count))
        else { return Flow::Continue; };
    let Some((slot, mut stack)) = candidates.into_iter().nth(pick as usize)
        else { return Flow::Continue; };
    let spent = repair(&mut stack, event.amount);
    if spent > 0 && player.set_inventory_slot(host, slot, Some(stack)).is_ok() {
        event.amount -= spent;
    }
    Flow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipped_slots_are_excluded_from_fallback() {
        assert!(equipped(2, 2));
        assert!(equipped(36, 2));
        assert!(equipped(40, 2));
        assert!(!eligible(2, 2, false));
        assert!(eligible(1, 2, false));
        assert!(!eligible(10, 2, true));
        assert!(eligible(8, 2, true));
        assert!(!eligible(36, 2, false));
    }
}
