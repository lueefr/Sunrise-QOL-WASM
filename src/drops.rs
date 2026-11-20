//! Shulker Drops Two retains the pre-spawn loot hook; death-specific item drops
//! use separate post-death world hooks to preserve the original event order.
use mod_api::{Host, Identifier, ItemStack, LootDropsEvent, ServerHost};
use crate::settings::{amount, flag, on};

fn add(host: &dyn Host, drops: &mut Vec<ItemStack>, id: &str, count: u32) {
    let Ok(name) = Identifier::parse(id) else { return; };
    let Some(item) = host.item_key(&name) else { return; };
    if let Some(stack) = ItemStack::new(item, count) {
        drops.push(stack);
    }
}

pub fn on_loot(host: &mut dyn ServerHost, event: &mut LootDropsEvent<'_>) {
    if event.context.used || event.context.damage_source.is_none() { return; }
    let Some((_, kind)) = event.context.entity else { return; };
    match kind {
        "minecraft:shulker" if on(host, "shulker_drops_two") => {
            let id = "minecraft:shulker_shell";
            let count = amount(host, "drop_count", 2., 1., 64.) as u32;
            // The original changes every shell item entity to exactly the
            // configured count (even when its existing stack was larger).
            // The loot hook acts before item entities spawn in Sunrise.
            let mut found = false;
            for stack in event.drops.iter_mut() {
                if stack.item().id(host) == id {
                    found = true;
                    let _ = stack.set_count(count);
                }
            }
            if !found && flag(host, "always_drop", false) {
                add(host, event.drops, id, count);
            }
        }
        _ => {}
    }
}
