//! Right Click Harvest, independently ported from JamCoreModding's MIT-licensed
//! mod. Keep crop interaction distinct from Tree Harvester and Leaf Decay.
//! The game owns item loot tables, Fortune and permissions; the WASM callback
//! requests the same block drops, restores the original crop properties and
//! leaves the normal game to synchronize the authoritative block.
use crate::settings::{flag, on, saved_value};
use mod_api::{
    BlockFace, BlockState, BlockUpdateFlags, EquipmentSlot, Flow, Hand,
    Identifier, InteractionResult, ItemKey, ItemStack, LevelRef,
    RightClickBlockEvent, ServerHost, StatType,
};
use std::sync::atomic::{AtomicU64, Ordering};

static XP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Crop {
    Age,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    None,
    Low,
    Mid,
    High,
}

fn is_hoe(host: &dyn ServerHost, stack: &ItemStack) -> bool {
    let item = stack.item();
    let id = item.id(host);
    id.ends_with("_hoe") || tagged_item(host, item, "minecraft:hoes")
        || tagged_item(host, item, "rightclickharvest:low_tier_hoes")
        || tagged_item(host, item, "rightclickharvest:mid_tier_hoes")
        || tagged_item(host, item, "rightclickharvest:high_tier_hoes")
}

fn tagged_item(host: &dyn ServerHost, item: ItemKey, id: &str) -> bool {
    let Ok(identifier) = Identifier::parse(id) else { return false; };
    host.tags().key(&identifier).is_some_and(|tag| {
        host.tags().is_in_tag(item, tag) == Some(true)
    })
}

fn tier(host: &dyn ServerHost, stack: &ItemStack) -> Tier {
    let item = stack.item();
    let id = item.id(host);
    if tagged_item(host, item, "rightclickharvest:high_tier_hoes")
        || id == "minecraft:netherite_hoe" {
        Tier::High
    } else if tagged_item(host, item, "rightclickharvest:mid_tier_hoes")
        || matches!(id.as_str(), "minecraft:golden_hoe" | "minecraft:diamond_hoe") {
        Tier::Mid
    } else if tagged_item(host, item, "rightclickharvest:low_tier_hoes")
        || matches!(id.as_str(), "minecraft:iron_hoe" | "minecraft:copper_hoe") {
        Tier::Low
    } else {
        Tier::None
    }
}

fn positions(pos: [i32; 3], tier: Tier) -> Vec<[i32; 3]> {
    let radius: i32 = match tier {
        Tier::None => 0,
        Tier::Low | Tier::Mid => 1,
        Tier::High => 2,
    };
    let mut result = Vec::with_capacity(25);
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if dx == 0 && dz == 0 { continue; }
            if matches!(tier, Tier::Low | Tier::High) && dx.abs() + dz.abs() > radius {
                continue;
            }
            if let (Some(x), Some(z)) = (pos[0].checked_add(dx), pos[2].checked_add(dz)) {
                result.push([x, pos[1], z]);
            }
        }
    }
    result.push(pos);
    result
}

fn prop<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let (_, tail) = state.split_once('[')?;
    let inside = tail.strip_suffix(']')?;
    inside.split(',').filter_map(|part| part.split_once('='))
        .find_map(|(name, value)| (name == key).then_some(value))
}

fn age_reset(state: &str) -> Option<String> {
    let (name, values) = state.split_once('[')?;
    let values = values.strip_suffix(']')?;
    let mut found = false;
    let result = values.split(',').map(|entry| {
        if entry.starts_with("age=") {
            found = true;
            "age=0".to_owned()
        } else {
            entry.to_owned()
        }
    }).collect::<Vec<_>>();
    found.then(|| format!("{name}[{}]", result.join(",")))
}

fn vanilla_crop(id: &str) -> bool {
    matches!(id,
        "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes"
        | "minecraft:beetroots" | "minecraft:nether_wart"
        | "minecraft:cocoa" | "minecraft:torchflower_crop")
}

fn mature(host: &dyn ServerHost, level: LevelRef, pos: [i32; 3],
    state: BlockState, id: &str) -> Option<(Crop, Option<BlockState>)> {
    if matches!(id, "minecraft:sugar_cane" | "minecraft:cactus") {
        return Some((Crop::Column, None));
    }
    let raw = host.block_state_string(state)?;
    let current = prop(&raw, "age")?.parse::<u32>().ok()?;
    let age = host.block_state_definition(state.block())?.into_iter()
        .find(|field| field.name == "age")?;
    let maximum = age.values.iter().filter_map(|value| value.parse::<u32>().ok()).max()?;
    if maximum == 0 || current < maximum { return None; }

    // Keep the original's CropBlock/Cocoa/NetherWart scope. Unknown modded
    // age-based blocks must live on farmland so that fire or decorative
    // age-bearing blocks are not mistaken for a crop.
    if !vanilla_crop(id) {
        if id.starts_with("minecraft:") { return None; }
        let below = [pos[0], pos[1].checked_sub(1)?, pos[2]];
        let ground = level.block(host, below)?.id(host);
        if !ground.contains("farmland") { return None; }
    }
    let reset = age_reset(&raw).and_then(|text| host.block_state_id(&text))?;
    Some((Crop::Age, Some(reset)))
}

fn replant_item(crop: &str) -> &str {
    match crop {
        "minecraft:wheat" => "minecraft:wheat_seeds",
        "minecraft:beetroots" => "minecraft:beetroot_seeds",
        "minecraft:carrots" => "minecraft:carrot",
        "minecraft:potatoes" => "minecraft:potato",
        "minecraft:nether_wart" => "minecraft:nether_wart",
        "minecraft:cocoa" => "minecraft:cocoa_beans",
        "minecraft:torchflower_crop" => "minecraft:torchflower_seeds",
        _ => crop,
    }
}

fn subtract_replant(host: &dyn ServerHost, crop: &str, drops: &mut Vec<ItemStack>) {
    let target = replant_item(crop);
    let index = drops.iter().position(|stack| stack.item().id(host) == target)
        .or_else(|| (!crop.starts_with("minecraft:")).then(|| {
            drops.iter().position(|stack| {
                let id = stack.item().id(host);
                id.contains("seed") || id == crop
            })
        }).flatten());
    if let Some(index) = index {
        if drops[index].count() <= 1 {
            drops.remove(index);
        } else {
            let count = drops[index].count() - 1;
            let _ = drops[index].set_count(count);
        }
    }
}

fn hunger_modifier(setting: Option<String>) -> f32 {
    match setting.as_deref() {
        Some("Low") => 0.5,
        Some("Normal") => 1.0,
        Some("High") => 2.0,
        _ => 0.0,
    }
}

fn exp_setting(setting: Option<String>) -> &'static str {
    match setting.as_deref() {
        Some("Cost") => "Cost",
        Some("Reward") => "Reward",
        _ => "None",
    }
}

fn xp_points() -> u32 {
    // The public hook cannot borrow the original player's Java Random.
    // Preserve the source's 0..=2 distribution, with a bounded guest stream.
    let n = XP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut n = n ^ 0x9e3779b97f4a7c15;
    n ^= n >> 30;
    n = n.wrapping_mul(0xbf58476d1ce4e5b9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94d049bb133111eb);
    ((n ^ (n >> 31)) % 3) as u32
}

fn xp_needed(level: i32) -> i32 {
    if level >= 30 { 9 * level - 158 }
    else if level >= 15 { 5 * level - 38 }
    else { 2 * level + 7 }
}

fn pay_xp(host: &mut dyn ServerHost, player: mod_api::PlayerRef, cost: i32) -> bool {
    if cost == 0 { return true; }
    let Some(old) = player.experience(host) else { return false; };
    let Some(needed) = player.xp_needed_for_next_level(host) else { return false; };
    let points = ((old.progress * needed as f32).round() as i32).max(0);
    let mut new_level = old.level;
    let mut new_points = points - cost;
    while new_points < 0 && new_level > 0 {
        new_level -= 1;
        new_points += xp_needed(new_level);
    }
    if new_points < 0 || old.total < cost { return false; }
    // Three setters replace one vanilla giveExperiencePoints(-N); if any
    // part cannot write, do not initiate a crop harvest.
    player.set_experience_levels(host, new_level)
        && player.set_experience_points(host, new_points)
        && player.set_total_experience(host, old.total - cost)
}

fn harvest_at(host: &mut dyn ServerHost, level: LevelRef,
    player: mod_api::PlayerRef, pos: [i32; 3], original: [i32; 3],
    held: Option<&ItemStack>, hoe: bool, creative: bool,
    hunger: f32, xp: &str) -> bool {
    let Some(state) = level.block_state(host, pos) else { return false; };
    let id = state.block().id(host);
    if host.block_in_tag(state.block(), "rightclickharvest:blacklist") == Some(true)
        || matches!(id.as_str(), "farmersdelight:tomatoes" | "farmersdelight:tomatoes_on_rope") {
        return false;
    }
    if pos != original &&
        (host.block_in_tag(state.block(), "rightclickharvest:radius_harvest_blacklist") == Some(true)
         || matches!(id.as_str(), "minecraft:cocoa" | "minecraft:cactus" | "minecraft:sugar_cane")) {
        return false;
    }
    let Some((kind, replanted)) = mature(host, level, pos, state, &id) else { return false; };
    let hoe_exempt = id == "minecraft:cocoa"
        || host.block_in_tag(state.block(), "rightclickharvest:hoe_never_required") == Some(true);
    if flag(host, "rchRequireHoe", false) && !hoe && !hoe_exempt { return false; }

    if let Some(held) = held {
        if player.may_use_item_at(host, pos, BlockFace::Up, held) != Some(true) {
            return false;
        }
    }
    if !creative && hunger > 0.0 &&
        player.food(host).is_none_or(|food| food.level <= 0) {
        return false;
    }
    if !creative && xp == "Cost" &&
        player.experience(host).is_none_or(|exp| exp.level < 2) {
        return false;
    }

    let position = match kind {
        Crop::Age => pos,
        Crop::Column => {
            let mut bottom = pos;
            while let Some(y) = bottom[1].checked_sub(1) {
                let below = [bottom[0], y, bottom[2]];
                if level.block(host, below).is_none_or(|block| block.id(host) != id) {
                    break;
                }
                bottom = below;
            }
            let Some(y) = bottom[1].checked_add(1) else { return false; };
            let upper = [bottom[0], y, bottom[2]];
            if level.block(host, upper).is_none_or(|block| block.id(host) != id) {
                return false;
            }
            upper
        }
    };
    if position != pos {
        if let Some(held) = held {
            if player.may_use_item_at(host, position, BlockFace::Up, held) != Some(true) {
                return false;
            }
        }
    }

    match kind {
        Crop::Column => {
            // Unlike direct set_block, the player break path keeps ordinary
            // loot, neighbour shape changes, damage and block-break hooks.
            if !player.destroy_block(host, position, false) { return false; }
        }
        Crop::Age => {
            let Some(mut drops) = level.block_drops(host, pos, held, Some(player.entity()))
                .ok().flatten() else { return false; };
            let Some(reset) = replanted else { return false; };
            if level.set_block(host, pos, reset,
                    BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS) != Ok(true) {
                return false;
            }
            subtract_replant(host, &id, &mut drops);
            for stack in drops {
                let _ = level.pop_resource(host, pos, stack);
            }
            let _ = player.award_stat(host, StatType::Mined, &id, 1);
            let replant = replant_item(&id);
            let _ = player.award_stat(host, StatType::Used, replant, 1);
        }
    }
    if let Some(held) = held {
        let _ = player.award_stat(host, StatType::Used, &held.item().id(host), 1);
    }
    if hoe && !creative {
        if let Some(living) = host.living(player.entity()) {
            let _ = living.hurt_and_break(host, EquipmentSlot::MainHand, 1);
        }
    }
    if !creative && hunger > 0.0 {
        let _ = player.add_exhaustion(host, 0.008 * hunger);
    }
    if !creative {
        match xp {
            "Cost" => { let _ = pay_xp(host, player, xp_points() as i32); }
            "Reward" => {
                let [x,y,z] = pos.map(|v| f64::from(v) + 0.5);
                let _ = level.award_experience(host, x, y, z, xp_points());
            }
            _ => {}
        }
    }
    true
}

pub fn click(host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "right_click_harvest") || event.click.hand != Hand::Main {
        return Flow::Continue;
    }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if player.is_spectator(host) != Some(false)
        || player.may_build(host) != Some(true)
        || player.is_secondary_use_active(host) == Some(true) {
        return Flow::Continue;
    }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }

    let held = event.click.stack.as_ref();
    let hoe = held.is_some_and(|stack| is_hoe(host, stack));
    let tier = if hoe && flag(host, "rchHarvestInRadius", true) {
        held.map(|stack| tier(host, stack)).unwrap_or(Tier::None)
    } else {
        Tier::None
    };
    let center = event.click.pos;
    let hunger = hunger_modifier(saved_value(host, "rchHungerLevel"));
    let xp = exp_setting(saved_value(host, "rchExperienceType"));
    let creative = player.is_creative(host) == Some(true);
    let mut harvested = 0;
    for pos in positions(center, tier) {
        if harvest_at(host, level, player, pos, center, held, hoe, creative, hunger, xp) {
            harvested += 1;
        }
    }
    if harvested == 0 { return Flow::Continue; }
    *event.cancellation_result = InteractionResult::SuccessServer;
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_hoe_shapes() {
        let center = [10, 64, 10];
        assert_eq!(positions(center, Tier::None), vec![center]);
        assert_eq!(positions(center, Tier::Low).len(), 5);
        assert_eq!(positions(center, Tier::Mid).len(), 9);
        assert_eq!(positions(center, Tier::High).len(), 13);
        assert!(positions(center, Tier::High).contains(&[12, 64, 10]));
        assert!(!positions(center, Tier::High).contains(&[12, 64, 11]));
    }

    #[test]
    fn keeps_modded_crop_properties_on_replant() {
        assert_eq!(age_reset("farmersdelight:rice[age=7,waterlogged=true]"),
            Some("farmersdelight:rice[age=0,waterlogged=true]".into()));
        assert_eq!(age_reset("minecraft:cocoa[age=2,facing=north]"),
            Some("minecraft:cocoa[age=0,facing=north]".into()));
        assert_eq!(age_reset("minecraft:wheat"), None);
    }

    #[test]
    fn matches_upstream_food_and_xp_modes() {
        assert_eq!(hunger_modifier(Some("Low".into())), 0.5);
        assert_eq!(hunger_modifier(Some("Normal".into())), 1.);
        assert_eq!(hunger_modifier(Some("High".into())), 2.);
        assert_eq!(hunger_modifier(None), 0.);
        assert_eq!(exp_setting(Some("Cost".into())), "Cost");
        assert_eq!(exp_setting(Some("Reward".into())), "Reward");
        assert_eq!(exp_setting(None), "None");
    }

    #[test]
    fn same_replant_mapping_as_vanilla_crops() {
        assert_eq!(replant_item("minecraft:wheat"), "minecraft:wheat_seeds");
        assert_eq!(replant_item("minecraft:carrots"), "minecraft:carrot");
        assert_eq!(replant_item("minecraft:nether_wart"), "minecraft:nether_wart");
    }
}
