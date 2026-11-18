//! Client-side enchantment tooltip additions. Only genuine saved enchantments
//! trigger cap markers; language entries are not replaced when already defined.
use crate::settings::{amount, flag, on, saved_value};
use mod_api::{ClientHost, ComponentId, ItemResource, ItemStack, Tag, Text, TextContent};
use std::collections::BTreeSet;

const MAX_TEXT_NODES: usize = 512;

fn level(value: &Tag) -> Option<u32> {
    match value {
        Tag::Byte(number) if *number > 0 => Some(*number as u32),
        Tag::Short(number) if *number > 0 => Some(*number as u32),
        Tag::Int(number) if *number > 0 => Some(*number as u32),
        Tag::Long(number) if *number > 0 => u32::try_from(*number).ok(),
        _ => None,
    }
}

fn enchantments(host: &dyn ClientHost, stack: &ItemStack) -> Vec<(String, u32)> {
    let mut found = Vec::new();
    for name in ["minecraft:enchantments", "minecraft:stored_enchantments"] {
        let Some(id) = ComponentId::new(name) else { continue; };
        let Some(Tag::Compound(levels)) = ItemResource::of(stack).value(host, &id) else { continue; };
        for (name, value) in levels {
            if let Some(level) = level(&value) {
                found.push((name.to_string_lossy(), level));
            }
        }
    }
    found
}

fn tr_key(id: &str) -> Option<String> {
    let (namespace, path) = id.split_once(':')?;
    Some(format!("enchantment.{namespace}.{}", path.replace('/', ".")))
}

fn has_key(text: &Text, key: &str) -> bool {
    let mut queue = vec![text];
    let mut checked = 0;
    while let Some(text) = queue.pop() {
        checked += 1;
        if checked > MAX_TEXT_NODES { return false; }
        if matches!(&text.content, TextContent::Translatable { key: t, .. } if t == key) {
            return true;
        }
        queue.extend(text.children.iter());
    }
    false
}

fn color(index: usize) -> &'static str {
    const COLORS: [&str; 16] = [
        "black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple",
        "gold", "gray", "dark_gray", "blue", "green", "aqua", "red",
        "light_purple", "yellow", "white",
    ];
    COLORS[index.min(15)]
}

fn cap_mark(host: &dyn ClientHost, stack: &ItemStack, lines: &mut [(Text, u32)]) {
    if !on(host, "enchantment_level_indicator") { return; }
    let levels = enchantments(host, stack);
    if levels.is_empty() { return; }
    let mut capped = BTreeSet::new();
    for info in host.enchantments() {
        if info.max_level == 0 { continue; }
        if levels.iter().any(|(id, level)| id == &info.id && *level >= info.max_level) {
            if let Some(key) = tr_key(&info.id) { capped.insert(key); }
        }
    }
    if capped.is_empty() { return; }
    let symbol = saved_value(host, "cap_symbol").unwrap_or_else(|| "★".to_owned());
    if symbol.is_empty() || symbol.len() > 24 { return; }
    let color = color(amount(host, "cap_color", 6., 0., 15.) as usize);
    for (text, _) in lines.iter_mut() {
        if capped.iter().any(|key| has_key(text, key)) {
            let mut suffix = Text::literal(format!(" {symbol}"));
            suffix.style.color = Some(color.to_owned());
            text.children.push(suffix);
        }
    }
}

fn roman(mut n: u32) -> Option<String> {
    if !(1..3999).contains(&n) { return None; }
    const PARTS: &[(u32, &str)] = &[
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"),
        (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"),
        (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut text = String::new();
    for (value, letters) in PARTS {
        while n >= *value { n -= *value; text.push_str(letters); }
    }
    Some(text)
}

fn replacement(host: &dyn ClientHost, key: &str) -> Option<String> {
    let (number, is_potion) = if let Some(value) = key.strip_prefix("enchantment.level.") {
        (value.parse::<u32>().ok()?, false)
    } else if let Some(value) = key.strip_prefix("potion.potency.") {
        if !flag(host, "patch_potions", true) { return None; }
        (value.parse::<u32>().ok()?.checked_add(1)?, true)
    } else { return None; };
    if number > 32767 || number == 0 { return None; }
    if !flag(host, "patch_override", false) && host.translate(key).is_some() {
        return None;
    }
    let roman_enabled = flag(host, if is_potion { "potion_roman" } else { "enchant_roman" }, false);
    if roman_enabled { Some(roman(number).unwrap_or_else(|| number.to_string())) }
    else { Some(number.to_string()) }
}

fn patch(host: &dyn ClientHost, text: &mut Text, processed: &mut usize) {
    *processed += 1;
    if *processed > MAX_TEXT_NODES { return; }
    if let TextContent::Translatable { key, .. } = &text.content {
        if let Some(value) = replacement(host, key) {
            text.content = TextContent::Literal(value);
        }
    }
    for child in &mut text.children {
        if *processed >= MAX_TEXT_NODES { break; }
        patch(host, child, processed);
    }
}

pub fn tooltip(host: &dyn ClientHost, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if on(host, "ench_level_langpatch") {
        let mut processed = 0;
        for (text, _) in lines.iter_mut() {
            patch(host, text, &mut processed);
        }
    }
    cap_mark(host, stack, lines);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn romans_are_correct() {
        assert_eq!(roman(1).as_deref(), Some("I"));
        assert_eq!(roman(49).as_deref(), Some("XLIX"));
        assert_eq!(roman(255).as_deref(), Some("CCLV"));
        assert!(roman(0).is_none());
        assert!(roman(3999).is_none());
    }
    #[test]
    fn cap_names_use_enchantment_translation_namespace() {
        assert_eq!(tr_key("minecraft:sharpness").as_deref(), Some("enchantment.minecraft.sharpness"));
        assert_eq!(color(6), "gold");
        assert_eq!(color(15), "white");
    }
}
