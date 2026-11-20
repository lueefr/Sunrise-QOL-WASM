//! Higher-level /enchant for the command sender's own held item.
//! Only operators may use this path; normal vanilla levels still run through vanilla.
use crate::settings::{amount, on};
use mod_api::{
    ComponentId, Flow, Identifier, ItemResource, PlayerCommandEvent, ServerHost,
    ServerTickEvent, ServerTickPhase, Tag, Text,
};
use std::sync::Mutex;

const MAX_QUEUE: usize = 64;
const MAX_LEVEL: u32 = 255;

#[derive(Clone)]
struct Request {
    player: u128,
    enchantment: String,
    level: u32,
}

#[derive(Default)]
pub struct EnchantCommands {
    pending: Mutex<Vec<Request>>,
}

fn parse(line: &str) -> Option<(String, u32)> {
    let mut fields = line.split_ascii_whitespace();
    let command = fields.next()?;
    if command != "enchant" && command != "minecraft:enchant" { return None; }
    if fields.next()? != "@s" { return None; }
    let name = fields.next()?;
    let level = fields.next()?.parse::<u32>().ok()?;
    if fields.next().is_some() || !(6..=MAX_LEVEL).contains(&level) { return None; }
    let id = Identifier::parse(name).ok()?;
    Some((id.to_string(), level))
}

fn level_of(tag: &Tag) -> Option<u32> {
    match tag {
        Tag::Int(value) if *value > 0 => u32::try_from(*value).ok(),
        Tag::Short(value) if *value > 0 => u32::try_from(*value).ok(),
        Tag::Byte(value) if *value > 0 => u32::try_from(*value).ok(),
        _ => None,
    }
}

fn eligible(item: &str, enchant: &str) -> bool {
    let name = enchant.strip_prefix("minecraft:").unwrap_or(enchant);
    let weapon = item.ends_with("_sword") || item.ends_with("_axe");
    let tool = weapon || item.ends_with("_pickaxe") || item.ends_with("_shovel") || item.ends_with("_hoe");
    let armor = ["_helmet", "_chestplate", "_leggings", "_boots"].iter().any(|v| item.ends_with(v));
    match name {
        "unbreaking" | "mending" => item != "minecraft:air",
        "sharpness" | "smite" | "bane_of_arthropods" | "fire_aspect" | "knockback" | "looting" | "sweeping_edge" => weapon,
        "efficiency" | "fortune" | "silk_touch" => tool,
        "protection" | "fire_protection" | "blast_protection" | "projectile_protection" | "thorns" => armor,
        "respiration" | "aqua_affinity" => item.ends_with("_helmet"),
        "feather_falling" | "depth_strider" | "frost_walker" | "soul_speed" => item.ends_with("_boots"),
        "power" | "punch" | "flame" | "infinity" => item == "minecraft:bow",
        "quick_charge" | "piercing" | "multishot" => item == "minecraft:crossbow",
        "loyalty" | "riptide" | "channeling" | "impaling" => item == "minecraft:trident",
        "luck_of_the_sea" | "lure" => item == "minecraft:fishing_rod",
        "density" | "breach" | "wind_burst" => item == "minecraft:mace",
        _ => false,
    }
}

impl EnchantCommands {
    pub fn command(&self, host: &dyn mod_api::Host, event: &mut PlayerCommandEvent<'_>) -> Flow {
        if !on(host, "enchant_command") { return Flow::Continue; }
        let Some((enchantment, level)) = parse(event.command) else { return Flow::Continue; };
        let ceiling = amount(host, "unlimited_max_level", 255., 1., 255.) as u32;
        if level > ceiling {
            event.replies.push(Text::literal(format!("Sunrise QOL: maximum enchantment level is {ceiling}.")));
            return Flow::Cancel;
        }
        let Ok(mut pending) = self.pending.lock() else { return Flow::Continue; };
        if pending.len() >= MAX_QUEUE {
            event.replies.push(Text::literal("Sunrise QOL: command queue is full."));
            return Flow::Cancel;
        }
        pending.push(Request { player: event.player, enchantment, level });
        Flow::Cancel
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let requests = if let Ok(mut queue) = self.pending.lock() {
            std::mem::take(&mut *queue)
        } else { return; };
        for request in requests.into_iter().take(MAX_QUEUE) {
            self.apply(host, request);
        }
    }

    fn apply(&self, host: &mut dyn ServerHost, request: Request) {
        if !on(host, "enchant_command") { return; }
        let Some(player) = host.player(request.player) else { return; };
        if player.permission_level(host).unwrap_or(0) < 2 {
            let _ = player.tell(host, Text::literal("Sunrise QOL: you need operator permission (level 2)."));
            return;
        }
        let Some(slot) = player.selected_slot(host).and_then(|v| u32::try_from(v).ok()) else { return; };
        let Some(stack) = player.inventory_slots(host).and_then(|slots| {
            slots.into_iter().find(|(index, _)| *index == slot).map(|(_, held)| held)
        }) else { return; };
        let item = stack.item().id(host);
        if !eligible(&item, &request.enchantment) {
            let _ = player.tell(host, Text::literal("Sunrise QOL: this item/enchantment combination is unsupported."));
            return;
        }
        let Some(id) = ComponentId::new("minecraft:enchantments") else { return; };
        let mut levels = match ItemResource::of(&stack).value(host, &id) {
            Some(Tag::Compound(existing)) => existing,
            None => mod_api::Compound::new(),
            _ => return,
        };
        // Keep mutually-exclusive enchantments out, as vanilla /enchant does.
        if levels.keys().any(|name| {
            let Ok(other) = name.to_string() else { return true; };
            other != request.enchantment && !host.enchantments_compatible(&other, &request.enchantment)
        }) {
            let _ = player.tell(host, Text::literal("Sunrise QOL: incompatible enchantments."));
            return;
        }
        if levels.get(&request.enchantment.as_str().into()).and_then(level_of)
            .is_some_and(|current| current >= request.level) { return; }
        levels.insert(request.enchantment.as_str().into(), Tag::Int(request.level as i32));
        let Ok(result) = ItemResource::of(&stack).with_component(host, &id, Tag::Compound(levels)) else { return; };
        let Some(next) = result.to_stack(stack.count()) else { return; };
        if player.set_inventory_slot(host, slot, Some(next)).is_ok() {
            let _ = player.tell(host, Text::literal(format!(
                "Sunrise QOL: applied {} {}.", request.enchantment, request.level
            )));
        }
    }

    pub fn reset(&self) {
        if let Ok(mut queue) = self.pending.lock() { queue.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_extended_self_commands_are_intercepted() {
        assert_eq!(parse("enchant @s sharpness 6"), Some(("minecraft:sharpness".into(), 6)));
        assert_eq!(parse("minecraft:enchant @s minecraft:sharpness 255"), Some(("minecraft:sharpness".into(), 255)));
        assert_eq!(parse("enchant @a sharpness 6"), None);
        assert_eq!(parse("enchant @s sharpness 5"), None);
        assert_eq!(parse("enchant @s sharpness 256"), None);
        assert!(eligible("minecraft:diamond_sword", "minecraft:sharpness"));
        assert!(!eligible("minecraft:stone", "minecraft:sharpness"));
    }
}
