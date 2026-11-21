//! Unbreakable Tools: protect all registered damageable gear from future wear.
//! Items on the legacy allowlist retain normal wear for compatibility.
//! Custom-item coverage depends on registrar support for tag Part attachment;
//! existing damage is deliberately unchanged (the hook controls future wear).
use crate::settings::{on, saved_value};
use mod_api::{DamageItemEvent, Flow, Identifier, ItemKey, ItemPart, Registrar, ServerHost};

#[derive(Clone)]
struct Protect;

fn is_allowed(raw: &str, id: &str, mut has_tag: impl FnMut(&str) -> bool) -> bool {
    raw.split(',').map(str::trim).filter(|entry| !entry.is_empty()).any(|entry| {
        if let Some(tag) = entry.strip_prefix('#') {
            Identifier::parse(tag).is_ok() && has_tag(tag)
        } else {
            Identifier::parse(entry).is_ok_and(|item| item.to_string() == id)
        }
    })
}

fn keeps_durability(host: &dyn ServerHost, item: ItemKey) -> bool {
    let Some(raw) = saved_value(host, "itemsWithDurability") else { return false; };
    let id = item.id(host);
    is_allowed(&raw, &id, |tag| {
        let Ok(tag) = Identifier::parse(tag) else { return false; };
        host.tags().key(&tag).is_some_and(|key| host.tags().is_in_tag(item, key) == Some(true))
    })
}

#[mod_api::sunrise_part]
impl ItemPart for Protect {
    fn damage_item(&self, host: &mut dyn ServerHost, event: &mut DamageItemEvent<'_>) -> Flow {
        if !on(host, "what_durability") { return Flow::Continue; }
        // Tag-bound parts can be reused by different items: inspect the event stack.
        if keeps_durability(host, event.stack.item()) { return Flow::Continue; }
        // Zero wear explicitly and cancel the normal durability update.
        event.amount = 0;
        Flow::Cancel
    }
}

const COMMON: &[&str] = &[
    "minecraft:bow", "minecraft:crossbow", "minecraft:shield", "minecraft:trident",
    "minecraft:mace", "minecraft:elytra", "minecraft:fishing_rod", "minecraft:shears",
    "minecraft:flint_and_steel", "minecraft:brush",
    "minecraft:carrot_on_a_stick", "minecraft:warped_fungus_on_a_stick",
];
const MATERIALS: &[&str] = &["wooden", "stone", "copper", "iron", "golden", "diamond", "netherite"];
const TOOLS: &[&str] = &["sword", "spear", "pickaxe", "axe", "shovel", "hoe"];
const ARMOR: &[&str] = &["helmet", "chestplate", "leggings", "boots"];
const TAGS: &[&str] = &[
    "minecraft:axes", "minecraft:pickaxes", "minecraft:shovels", "minecraft:hoes",
    "minecraft:swords", "minecraft:enchantable/durability",
    "minecraft:enchantable/mending", "minecraft:enchantable/weapon",
    "minecraft:enchantable/armor", "minecraft:enchantable/bow",
    "minecraft:enchantable/crossbow", "minecraft:enchantable/fishing",
    "minecraft:enchantable/trident",
];

pub fn register(r: &mut dyn Registrar) {
    for &name in COMMON {
        let item = r.item_key(name);
        r.item_part(item, Protect);
    }
    for &material in MATERIALS {
        for &tool in TOOLS {
            let id = format!("minecraft:{material}_{tool}");
            let item = r.item_key(&id);
            r.item_part(item, Protect);
        }
    }
    for material in ["leather", "chainmail", "copper", "iron", "golden", "diamond", "netherite"] {
        for &piece in ARMOR {
            let id = format!("minecraft:{material}_{piece}");
            let item = r.item_key(&id);
            r.item_part(item, Protect);
        }
    }
    let item = r.item_key("minecraft:turtle_helmet");
    r.item_part(item, Protect);
    // Tag attachments supplement explicitly registered items with modded tools.
    // A single ItemPart type may resolve multiple matching attachments; cancelling
    // remains idempotent and only affects the current wear operation.
    for tag in TAGS { r.item_part_tag(tag, Protect); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allowlist_keeps_wear_for_ids_and_tags() {
        assert!(!is_allowed("", "minecraft:bow", |_| false));
        assert!(is_allowed("minecraft:bow", "minecraft:bow", |_| false));
        assert!(is_allowed("bow", "minecraft:bow", |_| false));
        assert!(!is_allowed("minecraft:bow", "minecraft:elytra", |_| false));
        assert!(is_allowed("#minecraft:trimmable_armor", "minecraft:diamond_helmet", |tag| tag == "minecraft:trimmable_armor"));
        assert!(!is_allowed("#minecraft:trimmable_armor", "minecraft:diamond_helmet", |_| false));
        assert!(!is_allowed("bogus::name", "minecraft:bow", |_| true));
    }
    #[test]
    fn registers_vanilla_and_minecraft_tags() {
        assert_eq!(MATERIALS.len() * TOOLS.len() + 7 * ARMOR.len() + COMMON.len() + 1, 83);
        assert!(TAGS.contains(&"minecraft:enchantable/durability"));
    }
}
