//! Loot-table hooks. Preserve all unrelated vanilla and other-mod drops.
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
        "minecraft:wither_skeleton" if on(host, "always_wither_skull") => {
            let id = "minecraft:wither_skeleton_skull";
            if !event.drops.iter().any(|stack| stack.item().id(host) == id) {
                add(host, event.drops, id, 1);
            }
        }
        "minecraft:ender_dragon" if on(host, "dragon_drops_elytra") => {
            let id = "minecraft:elytra";
            if !event.drops.iter().any(|stack| stack.item().id(host) == id) {
                add(host, event.drops, id, 1);
            }
        }
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
