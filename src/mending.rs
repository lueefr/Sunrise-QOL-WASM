//! Inventory Mending spends experience that vanilla was about to grant the player.
//! Vanilla has already repaired equipped items before the XP-change event runs.
use crate::settings::{flag, on};
use mod_api::{ComponentId, Flow, ItemResource, JavaString, PlayerXpChangeEvent, ServerHost, Tag};

fn has_mending(host: &dyn ServerHost, stack: &mod_api::ItemStack) -> bool {
    let Some(id) = ComponentId::new("minecraft:enchantments") else { return false; };
    let Some(Tag::Compound(levels)) = ItemResource::of(stack).value(host, &id) else { return false; };
    levels.get(&JavaString::from("minecraft:mending")).is_some()
}

pub fn on_xp(host: &mut dyn ServerHost, event: &mut PlayerXpChangeEvent<'_>) -> Flow {
    if event.amount <= 0 || !on(host, "inventory_mending") { return Flow::Continue; }
    let Some(player) = host.player(event.player) else { return Flow::Continue; };
    let Some(slots) = player.inventory_slots(host) else { return Flow::Continue; };
    let hotbar_only = flag(host, "mending_hotbar_only", false);
    let mut xp = event.amount;
    for (slot, mut stack) in slots {
        if xp <= 0 { break; }
        if slot > 35 || (hotbar_only && slot > 8) || stack.damage() == 0
            || !has_mending(host, &stack) { continue; }
        let needed = stack.damage().div_ceil(2);
        let cost = needed.min(xp as u32);
        if cost == 0 { continue; }
        stack.set_damage(stack.damage().saturating_sub(cost.saturating_mul(2)));
        if player.set_inventory_slot(host, slot, Some(stack)).is_ok() {
            xp -= cost as i32;
        }
    }
    event.amount = xp;
    Flow::Continue
}

#[cfg(test)]
mod tests {
    #[test]
    fn two_durability_per_xp_with_round_up() {
        assert_eq!(1_u32.div_ceil(2), 1);
        assert_eq!(9_u32.div_ceil(2), 5);
        assert_eq!(20_u32.div_ceil(2), 10);
    }
}
