//! Cycle Paintings: cycle a painting through variants of the exact same size.
//! The server data-pack tag and variant metadata are authoritative; nested tags are skipped.
use crate::settings::on;
use mod_api::{
    Compound, EntityInteractEvent, Identifier, InteractionResult, PackType,
    ServerHost, Tag,
};

const MAX_VARIANTS: usize = 128;

fn painting_id(nbt: &Compound) -> Option<String> {
    match nbt.get(&"variant".into()) {
        Some(Tag::String(name)) => Some(name.to_string()),
        _ => None,
    }
}

fn variant_size(host: &dyn ServerHost, id: &str) -> Option<(u64, u64)> {
    let (namespace, name) = id.split_once(':')?;
    let path = Identifier::parse(&format!("{namespace}:painting_variant/{name}.json")).ok()?;
    let bytes = host.resource(PackType::ServerData, &path)?;
    let data: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let width = data.get("width")?.as_u64()?;
    let height = data.get("height")?.as_u64()?;
    (width > 0 && height > 0 && width <= 16 && height <= 16).then_some((width, height))
}

fn variants(host: &dyn ServerHost, current: &str) -> Vec<String> {
    let Some(path) = Identifier::parse("minecraft:tags/painting_variant/placeable.json").ok() else {
        return Vec::new();
    };
    let Some(bytes) = host.resource(PackType::ServerData, &path) else { return Vec::new(); };
    let Ok(data) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return Vec::new(); };
    let Some(entries) = data.get("values").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };
    let Some(size) = variant_size(host, current) else { return Vec::new(); };
    let mut options = Vec::new();
    for entry in entries.iter().take(MAX_VARIANTS) {
        let name = entry.as_str().or_else(|| entry.get("id").and_then(serde_json::Value::as_str));
        let Some(name) = name else { continue; };
        if name.starts_with('#') || name.len() > 128 { continue; }
        if variant_size(host, name) != Some(size) { continue; }
        if !options.iter().any(|old| old == name) {
            options.push(name.to_owned());
        }
    }
    options
}

fn next_variant(options: &[String], current: &str, reverse: bool) -> Option<&str> {
    if options.len() < 2 { return None; }
    let at = options.iter().position(|name| name == current)?;
    let next = if reverse {
        (at + options.len() - 1) % options.len()
    } else {
        (at + 1) % options.len()
    };
    options.get(next).map(String::as_str)
}

pub fn interact(host: &mut dyn ServerHost, event: &mut EntityInteractEvent<'_>) -> Option<InteractionResult> {
    if !on(host, "cycle_paintings") || event.kind != "minecraft:painting" {
        return None;
    }
    let stack = event.stack.as_ref()?;
    if stack.item().id(host) != "minecraft:painting" { return None; }
    let player = host.players().acting_player()?;
    let dimension = player.level(host)?.dimension().id(host)?;
    let reverse = player.is_secondary_use_active(host) == Some(true);
    let painting = host.entity(event.entity)?;
    let mut nbt = painting.nbt(host)?;
    let current = painting_id(&nbt)?;
    let options = variants(host, &current);
    let next = next_variant(&options, &current, reverse)?;
    let pos = painting.position(host)?;
    nbt.insert("variant".into(), Tag::String(next.into()));
    // Never copy an entity UUID into another live entity.
    nbt.remove(&"UUID".into());
    nbt.remove(&"UUIDMost".into());
    nbt.remove(&"UUIDLeast".into());
    let created = host.summon(&dimension, "minecraft:painting", pos, nbt)?;
    if painting.discard(host) {
        Some(InteractionResult::SuccessServer)
    } else {
        if let Some(spawned) = host.entity(created) { let _ = spawned.discard(host); }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycles_forward_and_reverse_and_keeps_size() {
        let items = vec!["minecraft:a".to_owned(), "minecraft:b".to_owned(), "minecraft:c".to_owned()];
        assert_eq!(next_variant(&items, "minecraft:b", false), Some("minecraft:c"));
        assert_eq!(next_variant(&items, "minecraft:a", true), Some("minecraft:c"));
        assert_eq!(next_variant(&items, "minecraft:z", false), None);
    }
}
