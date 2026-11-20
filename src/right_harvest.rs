//! Right Click Harvest, independently ported from JamCoreModding's MIT-licensed
//! mod. Keep crop interaction distinct from Tree Harvester and Leaf Decay.
//! The game owns item loot tables, Fortune and permissions; the WASM callback
//! requests the same block drops, restores the original crop properties and
//! leaves the normal game to synchronize the authoritative block.
use crate::settings::{flag, on, saved_value};
use mod_api::{
    BlockFace, BlockState, BlockUpdateFlags, ClientHost, ComponentId, EquipmentSlot, Flow,
    Hand, Host, Identifier, InteractionResult, ItemKey, ItemResource, ItemStack, LevelRef,
    RightClickBlockEvent, ServerHost, SoundPlay, SoundSource, StatType, Tag, Text,
};

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

fn is_hoe(host: &dyn Host, stack: &ItemStack) -> bool {
    let item = stack.item();
    tagged_item(host, item, "minecraft:hoes")
        || tagged_item(host, item, "rightclickharvest:low_tier_hoes")
        || tagged_item(host, item, "rightclickharvest:mid_tier_hoes")
        || tagged_item(host, item, "rightclickharvest:high_tier_hoes")
        || ComponentId::new("minecraft:block_transformer")
            .and_then(|component| ItemResource::of(stack).value(host, &component))
            .is_some_and(|tag| match tag {
                Tag::String(value) => value.to_string().ok().as_deref() == Some("minecraft:hoe"),
                _ => false,
            })
}

fn tagged_item(host: &dyn Host, item: ItemKey, id: &str) -> bool {
    let Ok(identifier) = Identifier::parse(id) else { return false; };
    host.tags().key(&identifier).is_some_and(|tag| {
        host.tags().is_in_tag(item, tag) == Some(true)
    })
}

fn tier(host: &dyn Host, stack: &ItemStack) -> Tier {
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
    // Upstream's low tier traverses NORTH, EAST, SOUTH, WEST in this order,
    // not the nested-coordinate loop used for middle and high tiers.
    if tier == Tier::Low {
        let mut result = Vec::with_capacity(5);
        for [dx, dz] in [[0, -1], [1, 0], [0, 1], [-1, 0]] {
            if let (Some(x), Some(z)) = (pos[0].checked_add(dx), pos[2].checked_add(dz)) {
                result.push([x, pos[1], z]);
            }
        }
        result.push(pos);
        return result;
    }
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

fn harvest_positions(
    host: &dyn Host,
    level: LevelRef,
    center: [i32; 3],
    held: Option<&ItemStack>,
    radius: bool,
) -> Vec<[i32; 3]> {
    let Some(stack) = held.filter(|stack| radius && is_hoe(host, stack)) else {
        return vec![center];
    };
    let Some(state) = level.block_state(host, center) else { return vec![center]; };
    let id = state.block().id(host);
    // The upstream's recursive radius is entered only from a mature
    // age-based crop that is not blacklisted for radius harvesting.
    if host.block_in_tag(state.block(), "rightclickharvest:blacklist") == Some(true)
        || host.block_in_tag(state.block(), "rightclickharvest:radius_harvest_blacklist") == Some(true)
        || !matches!(mature(host, level, center, state, &id), Some((Crop::Age, _))) {
        return vec![center];
    }
    positions(center, tier(host, stack))
}

fn prop<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let (_, tail) = state.split_once('[')?;
    let inside = tail.strip_suffix(']')?;
    inside.split(',').filter_map(|part| part.split_once('='))
        .find_map(|(name, value)| (name == key).then_some(value))
}

fn age_value(state: &str, age: u32) -> Option<String> {
    let (name, values) = state.split_once('[')?;
    let values = values.strip_suffix(']')?;
    let mut found = false;
    let result = values.split(',').map(|entry| {
        if entry.starts_with("age=") {
            found = true;
            format!("age={age}")
        } else {
            entry.to_owned()
        }
    }).collect::<Vec<_>>();
    found.then(|| format!("{name}[{}]", result.join(",")))
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

fn mature(host: &dyn Host, _level: LevelRef, _pos: [i32; 3],
    state: BlockState, id: &str) -> Option<(Crop, Option<BlockState>)> {
    if matches!(id, "minecraft:sugar_cane" | "minecraft:cactus") {
        return Some((Crop::Column, None));
    }
    let raw = host.block_state_string(state)?;
    let current = prop(&raw, "age")?.parse::<u32>().ok()?;
    // The pinned ServerHost exposes block_state_id, but not the metadata
    // definition API. Probe the next legal age to detect maturity, without
    // assuming vanilla-only level caps or discarding modded properties.
    let next = age_value(&raw, current.checked_add(1)?)?;
    if host.block_state_id(&next).is_some() { return None; }

    // CropBlock subclasses are not visible to the WASM host. Accept
    // registered non-vanilla blocks with a bounded age property and
    // let the exact upstream blacklist decide known exceptions; this
    // supports modded crops on custom soil and vertical growing systems.
    if id.starts_with("minecraft:") && !vanilla_crop(id) {
        return None;
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

fn subtract_replant(host: &dyn ServerHost, crop: &str,
    block: mod_api::BlockKey, drops: &mut Vec<ItemStack>) {
    // Block.asItem (and Item.getBlock) are the public SDK equivalents for
    // most crop clone-item relationships. They are more reliable than
    // guessing by substring for a modded crop with custom seed names.
    let target = if crop.starts_with("minecraft:") {
        Some(replant_item(crop).to_owned())
    } else {
        host.block_item(block).map(|item| item.id(host))
            .filter(|id| id != "minecraft:air")
    };
    let index = drops.iter().position(|stack| {
        target.as_deref() == Some(stack.item().id(host).as_str())
    }).or_else(|| drops.iter().position(|stack| {
        host.item_block(stack.item()) == Some(block)
    }));
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

fn xp_points(host: &mut dyn ServerHost, level: LevelRef) -> u32 {
    // Sample the game's authoritative random source instead of a shared
    // independent counter. The original samples Player.random, which
    // Sunrise's WASM API does not currently lend through ServerHost.
    level.with_random(host, |random| random.next_int_bound(3).clamp(0, 2) as u32)
        .unwrap_or(0)
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

fn column_top(host: &dyn Host, level: LevelRef, pos: [i32; 3],
    id: &str) -> Option<[i32; 3]> {
    let mut bottom = pos;
    while let Some(y) = bottom[1].checked_sub(1) {
        let below = [bottom[0], y, bottom[2]];
        if level.block(host, below).is_none_or(|block| block.id(host) != id) {
            break;
        }
        bottom = below;
    }
    let y = bottom[1].checked_add(1)?;
    let upper = [bottom[0], y, bottom[2]];
    (level.block(host, upper)?.id(host) == id).then_some(upper)
}

fn wear_needed(kind: Crop, previous: Option<u32>, current: Option<u32>) -> i32 {
    match (kind, previous, current) {
        (Crop::Age, Some(_), _) => 1,
        (Crop::Column, Some(before), Some(after)) if after <= before => 1,
        _ => 0,
    }
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
            let Some(upper) = column_top(host, level, pos, &id) else { return false; };
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

    // The normal player-break route can spend exhaustion and durability
    // itself. The original charges exactly one hoe wear and its chosen
    // hunger multiplier, so measure changes before applying any difference.
    let before_food = (kind == Crop::Column).then(|| player.food(host))
        .flatten().map(|food| food.exhaustion);
    let before_damage = held.filter(|_| hoe && !creative).map(|stack| stack.damage());
    let selected = if hoe && !creative { player.selected_slot(host) } else { None };
    let held_id = held.map(|stack| stack.item().id(host));

    match kind {
        Crop::Column => {
            // JamCoreModding's Block.getDrops runs even for creative players.
            // Sunrise's creative-mode player break suppresses normal loot,
            // so use the level's loot-aware destroy path for that mode.
            if creative {
                if level.destroy_block(host, position, true, held, false)
                    .ok().flatten().is_none() {
                    return false;
                }
                let _ = player.award_stat(host, StatType::Mined, &id, 1);
            } else {
                // Survival runs the player's real break path, including
                // permissions, block hooks and native tool wear.
                if !player.destroy_block(host, position, false) {
                    return false;
                }
            }
        }
        Crop::Age => {
            let Some(mut drops) = level.block_drops(host, pos, held, Some(player.entity()))
                .ok().flatten() else { return false; };
            let Some(reset) = replanted else { return false; };
            if level.set_block(host, pos, reset,
                    BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS) != Ok(true) {
                return false;
            }
            subtract_replant(host, &id, state.block(), &mut drops);
            for stack in drops {
                let _ = level.pop_resource(host, pos, stack);
            }
            let _ = player.award_stat(host, StatType::Mined, &id, 1);
            let replant = replant_item(&id);
            let _ = player.award_stat(host, StatType::Used, replant, 1);
        }
    }
    if let Some(held) = held {
        let tool_id = held.item().id(host);
        let _ = player.award_stat(host, StatType::Used, &tool_id, 1);
    }
    if hoe && !creative {
        let current = if kind == Crop::Column {
            player.inventory_slots(host).and_then(|slots| slots.into_iter()
                .find(|(slot, stack)| {
                    usize::try_from(*slot).ok() == selected
                        && held_id.as_deref() == Some(stack.item().id(host).as_str())
                })
                .map(|(_, stack)| stack.damage()))
        } else { None };
        if wear_needed(kind, before_damage, current) > 0 {
            if let Some(living) = host.living(player.entity()) {
                let _ = living.hurt_and_break(host, EquipmentSlot::MainHand, 1);
            }
        }
    }
    if !creative {
        if let Some(original) = before_food {
            // Cancel the player-break route's automatic exhaustion and
            // replace it with the source mod's 0.008 * level modifier.
            let _ = player.set_exhaustion(host, original + 0.008 * hunger);
        } else if hunger > 0.0 {
            let _ = player.add_exhaustion(host, 0.008 * hunger);
        }
    }
    if !creative {
        match xp {
            "Cost" => { let cost = xp_points(host, level) as i32; let _ = pay_xp(host, player, cost); }
            "Reward" => {
                let [x,y,z] = pos.map(|v| f64::from(v) + 0.5);
                let points = xp_points(host, level);
                let _ = level.award_experience(host, x, y, z, points);
            }
            _ => {}
        }
    }
    true
}

// The source mod plays the planting sound on the client during the click,
// and warns once when a bare hand cannot harvest because a hoe is required.
pub fn client_click(host: &mut dyn ClientHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !host.playing() || !on(host, "right_click_harvest")
        || event.click.hand != Hand::Main {
        return Flow::Continue;
    }
    let Some(player) = host.local_player() else { return Flow::Continue; };
    if player.is_spectator(host) != Some(false)
        || player.is_secondary_use_active(host) == Some(true) {
        return Flow::Continue;
    }
    let Some(level) = player.entity().level(host) else { return Flow::Continue; };
    let pos = event.click.pos;
    let Some(state) = level.block_state(host, pos) else { return Flow::Continue; };
    let id = state.block().id(host);
    if host.block_in_tag(state.block(), "rightclickharvest:blacklist") == Some(true)
        || matches!(id.as_str(), "farmersdelight:tomatoes" | "farmersdelight:tomatoes_on_rope") {
        return Flow::Continue;
    }
    let Some((kind, _)) = mature(host, level, pos, state, &id) else {
        return Flow::Continue;
    };
    if kind == Crop::Column && column_top(host, level, pos, &id).is_none() {
        return Flow::Continue;
    }
    if event.click.hit.face == [0, 1, 0] && kind == Crop::Column
        && event.click.stack.is_some_and(|stack| stack.item().id(host) == id) {
        return Flow::Continue;
    }
    let hoe = event.click.stack.is_some_and(|stack| is_hoe(host, stack));
    let exempt = id == "minecraft:cocoa"
        || host.block_in_tag(state.block(), "rightclickharvest:hoe_never_required") == Some(true);
    if flag(host, "rchRequireHoe", false) && !hoe && !exempt {
        if event.click.stack.is_none() && !flag(host, "rchHoeWarned", false) {
            let _ = host.chat_line(Text::literal(
                "Right Click Harvest requires a hoe to harvest this crop.                  Equip a hoe or disable Require Hoe in the mod configuration.                  This warning is shown only once.",
            ));
            let _ = crate::settings::save(host, "rchHoeWarned", "true");
        }
        return Flow::Continue;
    }
    let creative = player.is_creative(host) == Some(true);
    if !creative && hunger_modifier(saved_value(host, "rchHungerLevel")) > 0.
        && player.food(host).is_none_or(|food| food.level <= 0) {
        return Flow::Continue;
    }
    if !creative && exp_setting(saved_value(host, "rchExperienceType")) == "Cost"
        && player.experience(host).is_none_or(|exp| exp.level < 2) {
        return Flow::Continue;
    }
    // The original calls onBlockUse recursively for neighboring crops and
    // plays each successful planting sound at the player's location. The
    // clicked crop sounds last; sugar cane and cactus never expand a radius.
    let nearby = harvest_positions(
        host, level, pos, event.click.stack,
        flag(host, "rchHarvestInRadius", true),
    );
    let origin = player.entity().position(host)
        .unwrap_or_else(|| pos.map(|v| f64::from(v) + 0.5));
    let mut played = false;
    for target in nearby {
        let Some(state) = level.block_state(host, target) else { continue; };
        let id = state.block().id(host);
        if host.block_in_tag(state.block(), "rightclickharvest:blacklist") == Some(true)
            || (target != pos &&
                host.block_in_tag(state.block(), "rightclickharvest:radius_harvest_blacklist") == Some(true)) {
            continue;
        }
        let Some((kind, _)) = mature(host, level, target, state, &id) else { continue; };
        if kind == Crop::Column && (target != pos || column_top(host, level, target, &id).is_none()) {
            continue;
        }
        let exempt = id == "minecraft:cocoa"
            || host.block_in_tag(state.block(), "rightclickharvest:hoe_never_required") == Some(true);
        if flag(host, "rchRequireHoe", false) && !hoe && !exempt {
            continue;
        }
        let sound = if id == "minecraft:nether_wart" {
            "minecraft:item.nether_wart.plant"
        } else {
            "minecraft:item.crop.plant"
        };
        let _ = host.play_local_sound(
            &SoundPlay::at(sound, SoundSource::Players, origin), 0,
        );
        played = true;
    }
    if !played { return Flow::Continue; }
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
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

    let held = event.click.stack;
    // Clicking the top face with a matching sugar cane/cactus item is vanilla
    // placement, not harvesting. Preserve it as the original does.
    if event.click.hit.face == [0, 1, 0] {
        let id = level.block(host, event.click.pos).map(|block| block.id(host));
        if id.as_deref().is_some_and(|id| matches!(id,
                "minecraft:cactus" | "minecraft:sugar_cane"))
            && held.is_some_and(|stack| Some(stack.item().id(host)) == id) {
            return Flow::Continue;
        }
    }
    let hoe = held.is_some_and(|stack| is_hoe(host, stack));
    let center = event.click.pos;
    let positions = harvest_positions(
        host, level, center, held,
        flag(host, "rchHarvestInRadius", true),
    );
    let hunger = hunger_modifier(saved_value(host, "rchHungerLevel"));
    let xp = exp_setting(saved_value(host, "rchExperienceType"));
    let creative = player.is_creative(host) == Some(true);
    let mut harvested = 0;
    for pos in positions {
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
        assert_eq!(positions(center, Tier::Low), vec![
            [10, 64, 9], [11, 64, 10], [10, 64, 11], [9, 64, 10], center
        ]);
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
    fn exact_one_point_of_hoe_wear() {
        assert_eq!(wear_needed(Crop::Age, Some(9), None), 1);
        assert_eq!(wear_needed(Crop::Column, Some(9), Some(9)), 1);
        assert_eq!(wear_needed(Crop::Column, Some(9), Some(10)), 0);
        assert_eq!(wear_needed(Crop::Column, Some(9), None), 0);
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
    fn validates_next_age_without_losing_other_properties() {
        assert_eq!(age_value("mod:plant[age=4,ropecrop=true]", 5),
            Some("mod:plant[age=5,ropecrop=true]".into()));
        assert_eq!(age_value("minecraft:stone", 1), None);
    }

    #[test]
    fn same_replant_mapping_as_vanilla_crops() {
        assert_eq!(replant_item("minecraft:wheat"), "minecraft:wheat_seeds");
        assert_eq!(replant_item("minecraft:carrots"), "minecraft:carrot");
        assert_eq!(replant_item("minecraft:nether_wart"), "minecraft:nether_wart");
    }
}
