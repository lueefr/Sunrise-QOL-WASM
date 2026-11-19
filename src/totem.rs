//! Inventory Totem follows the item component checked by the original mod.
//! The ordinary held-item protection is always left to Sunrise's vanilla path.
//! The WASM API has no direct entry point for vanilla's death-protection effect executor.
use crate::settings::on;
use mod_api::{
    ActiveEffect, ComponentId, Flow, ItemResource, JavaString, LivingDeathEvent, ServerHost,
    StatType, Tag,
};

#[derive(Clone, Debug, PartialEq)]
struct Potion {
    id: String,
    duration: i32,
    amplifier: i32,
    ambient: bool,
    particles: bool,
    icon: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum DeathEffect {
    Clear,
    Apply(Vec<Potion>),
}

fn field<'a>(map: &'a mod_api::Compound, key: &str) -> Option<&'a Tag> {
    map.get(&JavaString::from(key))
}

fn text(map: &mod_api::Compound, key: &str) -> Option<String> {
    match field(map, key)? {
        Tag::String(value) => value.to_string().ok(),
        _ => None,
    }
}

fn integer(tag: &Tag) -> Option<i32> {
    match tag {
        Tag::Byte(value) => Some(i32::from(*value)),
        Tag::Short(value) => Some(i32::from(*value)),
        Tag::Int(value) => Some(*value),
        Tag::Long(value) => i32::try_from(*value).ok(),
        _ => None,
    }
}

fn number(map: &mod_api::Compound, key: &str, default: i32) -> Option<i32> {
    match field(map, key) {
        None => Some(default),
        Some(value) => integer(value),
    }
}

fn enabled(map: &mod_api::Compound, key: &str, default: bool) -> Option<bool> {
    match field(map, key) {
        None => Some(default),
        Some(Tag::Byte(0)) => Some(false),
        Some(Tag::Byte(1)) => Some(true),
        _ => None,
    }
}

fn probability(map: &mod_api::Compound) -> bool {
    match field(map, "probability") {
        None => true,
        Some(Tag::Float(value)) => f32::from_bits(*value) == 1.,
        Some(Tag::Double(value)) => f64::from_bits(*value) == 1.,
        Some(tag) => integer(tag) == Some(1),
    }
}

// Unsupported vanilla consume-effect types are rejected rather than applying
// an unrelated totem effect to a modded death-protection item.
fn decode(tag: &Tag) -> Option<Vec<DeathEffect>> {
    let Tag::Compound(root) = tag else { return None; };
    let Some(entries) = field(root, "death_effects") else { return Some(Vec::new()); };
    let Tag::List(entries) = entries else { return None; };
    let mut result = Vec::new();
    for entry in entries {
        let Tag::Compound(data) = entry else { return None; };
        match text(data, "type")?.as_str() {
            "minecraft:clear_all_effects" => result.push(DeathEffect::Clear),
            "minecraft:apply_effects" if probability(data) => {
                let Tag::List(items) = field(data, "effects")? else { return None; };
                let mut potions = Vec::new();
                for item in items {
                    let Tag::Compound(spec) = item else { return None; };
                    let duration = number(spec, "duration", 0)?;
                    let amplifier = number(spec, "amplifier", 0)?;
                    if duration < -1 || amplifier < 0 { return None; }
                    potions.push(Potion {
                        id: text(spec, "id")?,
                        duration,
                        amplifier,
                        ambient: enabled(spec, "ambient", false)?,
                        particles: enabled(spec, "show_particles", true)?,
                        icon: enabled(spec, "show_icon",
                            enabled(spec, "show_particles", true)?)?,
                    });
                }
                result.push(DeathEffect::Apply(potions));
            }
            _ => return None,
        }
    }
    Some(result)
}

fn bypasses(source: &str) -> bool {
    matches!(source, "minecraft:generic_kill" | "minecraft:out_of_world")
}

pub fn on_death(host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
    if event.kind != "minecraft:player" || !on(host, "inventory_totem")
        || bypasses(event.source.damage_type) {
        return Flow::Continue;
    }
    let Some(player) = host.players().server_players().into_iter().find(|player| {
        player.entity().id() == event.entity
    }) else { return Flow::Continue; };
    let Some(component) = ComponentId::new("minecraft:death_protection") else {
        return Flow::Continue;
    };
    let selected = player.selected_slot(host);
    let Some((slot, mut stack, effects)) = player.inventory_slots(host)
        .and_then(|slots| slots.into_iter().find_map(|(slot, stack)| {
            // Vanilla checks the held stack first, using its actual component
            // and effect executor; it must not be consumed a second time here.
            if slot == 40 || usize::try_from(slot).ok() == selected {
                return None;
            }
            let protection = ItemResource::of(&stack).value(host, &component)?;
            let effects = decode(&protection)?;
            Some((slot, stack, effects))
        })) else { return Flow::Continue; };

    // Resolve every potion effect before mutating either health or inventory.
    // No effect key or unsupported effect form may trigger a partial rescue.
    let mut resolved = Vec::new();
    for action in effects {
        match action {
            DeathEffect::Clear => resolved.push((true, Vec::new())),
            DeathEffect::Apply(potions) => {
                let mut active = Vec::new();
                for effect in potions {
                    let Some(key) = host.effect_key(&effect.id) else {
                        return Flow::Continue;
                    };
                    active.push(ActiveEffect::new(key, effect.duration,
                        effect.amplifier, effect.ambient, effect.particles, effect.icon));
                }
                resolved.push((false, active));
            }
        }
    }
    let Some(entity) = host.entity(event.entity) else { return Flow::Continue; };
    let Some(living) = host.living(entity) else { return Flow::Continue; };
    let original = stack.clone();
    let rest = if stack.count() == 1 { None } else {
        let next = stack.count() - 1;
        if stack.set_count(next).is_err() { return Flow::Continue; }
        Some(stack)
    };
    if player.set_inventory_slot(host, slot, rest).is_err() { return Flow::Continue; }
    if !living.set_health(host, 1.) {
        let _ = player.set_inventory_slot(host, slot, Some(original));
        return Flow::Continue;
    }
    for (clear, effects) in resolved {
        if clear { let _ = living.remove_all_effects(host); }
        for effect in effects {
            let _ = living.add_effect(host, &effect);
        }
    }
    // The statistic is public, but the vanilla USED_TOTEM criterion and byte-35
    // activation animation still require a dedicated public API operation.
    let _ = player.award_stat(host, StatType::Used, &original.item().id(host), 1);
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_damage_does_not_consume_an_inventory_item() {
        assert!(bypasses("minecraft:generic_kill"));
        assert!(bypasses("minecraft:out_of_world"));
        assert!(!bypasses("minecraft:fall"));
    }

    #[test]
    fn death_component_supports_effects_and_empty_payloads() {
        let empty = Tag::Compound(mod_api::Compound::new());
        assert_eq!(decode(&empty), Some(Vec::new()));
        let protection = Tag::compound([("death_effects", Tag::List(vec![
            Tag::compound([("type", Tag::String("minecraft:clear_all_effects".into()))]),
            Tag::compound([
                ("type", Tag::String("minecraft:apply_effects".into())),
                ("effects", Tag::List(vec![Tag::compound([
                    ("id", Tag::String("minecraft:regeneration".into())),
                    ("duration", Tag::Int(900)),
                    ("amplifier", Tag::Int(1)),
                ])])),
            ]),
        ]))]);
        let parsed = decode(&protection).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], DeathEffect::Clear);
        assert_eq!(parsed[1], DeathEffect::Apply(vec![Potion {
            id: "minecraft:regeneration".to_owned(), duration: 900,
            amplifier: 1, ambient: false, particles: true, icon: true,
        }]));
    }

    #[test]
    fn unknown_effect_never_falls_back_to_vanilla_totem() {
        let custom = Tag::compound([("death_effects", Tag::List(vec![
            Tag::compound([("type", Tag::String("example:custom".into()))]),
        ]))]);
        assert_eq!(decode(&custom), None);
    }
}
