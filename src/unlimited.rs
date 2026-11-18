//! Anvil merging above each enchantment's vanilla cap, up to the 255-level
//! ItemEnchantments representation. Does not patch the registry or allow
//! combinations vanilla rejected before AnvilUpdateEvent.
use crate::settings::{amount, on};
use mod_api::{AnvilUpdateEvent, ComponentId, Host, ItemResource, ItemStack, Tag};

fn read(host: &dyn Host, stack: &ItemStack) -> Option<Tag> {
    for name in ["minecraft:stored_enchantments", "minecraft:enchantments"] {
        let Some(id) = ComponentId::new(name) else { continue; };
        if let Some(value) = ItemResource::of(stack).value(host, &id) {
            return Some(value);
        }
    }
    None
}

fn level(tag: &Tag) -> Option<u32> {
    match tag {
        Tag::Byte(x) if *x > 0 => Some(*x as u32),
        Tag::Short(x) if *x > 0 => Some(*x as u32),
        Tag::Int(x) if *x > 0 => Some(*x as u32),
        Tag::Long(x) if *x > 0 => u32::try_from(*x).ok(),
        _ => None,
    }
}

pub fn anvil(host: &dyn Host, event: &mut AnvilUpdateEvent<'_>) {
    if !on(host, "unlimited_enchantments") { return; }
    let Some(ref output) = event.update.output else { return; };
    let Some(right) = &event.update.right else { return; };
    let (Some(Tag::Compound(left)), Some(Tag::Compound(other))) = (
        read(host, &event.update.left), read(host, right)
    ) else { return; };

    let is_book = output.item().id(host) == "minecraft:enchanted_book";
    let component = if is_book { "minecraft:stored_enchantments" } else { "minecraft:enchantments" };
    let Some(id) = ComponentId::new(component) else { return; };
    let Some(Tag::Compound(mut levels)) = ItemResource::of(output).value(host, &id) else { return; };
    let ceiling = amount(host, "unlimited_max_level", 255., 1., 255.) as u32;
    let mut changed = false;
    for (name, a) in &left {
        let Some(b) = other.get(name) else { continue; };
        let (Some(a), Some(b)) = (level(a), level(b)) else { continue; };
        if a == b && a < ceiling {
            let upgraded = a + 1;
            if levels.get(name).and_then(level).is_some_and(|current| current < upgraded) {
                levels.insert(name.clone(), Tag::Int(upgraded as i32));
                changed = true;
            }
        }
    }
    if !changed { return; }
    let Ok(resource) = ItemResource::of(output).with_component(host, &id, Tag::Compound(levels)) else { return; };
    let Some(stack) = resource.to_stack(output.count()) else { return; };
    event.update.output = Some(stack);
    // Anvil cost overrides apply only when the mod actually produced a stronger result.
    let max_cost = amount(host, "unlimited_max_anvil_cost", 39., 1., 10000.) as i32;
    if event.update.xp_cost > max_cost { event.update.xp_cost = max_cost; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interprets_saved_level_values() {
        assert_eq!(level(&Tag::Int(255)), Some(255));
        assert_eq!(level(&Tag::Int(0)), None);
        assert_eq!(level(&Tag::Int(3)), Some(3));
    }
}
