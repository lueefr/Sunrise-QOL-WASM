//! Extended Bone Meal: immediate crop maturity while sneaking, and vanilla-rate
//! growth for selected previously unsupported plants. The mod never creates meal.
use crate::settings::{flag, on};
use mod_api::{BlockUpdateFlags, BonemealEvent, Flow, HostSide, ServerHost};

fn flags() -> BlockUpdateFlags { BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS }

fn next_age(state: &str, max: u32, instant: bool) -> Option<String> {
    let (name, values) = state.split_once('[')?;
    let values = values.strip_suffix(']')?;
    let mut changed = false;
    let next = values.split(',').map(|part| {
        let Some((key, value)) = part.split_once('=') else { return part.to_owned(); };
        if key != "age" { return part.to_owned(); }
        let Some(age) = value.parse::<u32>().ok() else { return part.to_owned(); };
        if age >= max { return part.to_owned(); }
        changed = true;
        format!("age={}", if instant { max } else { age + 1 })
    }).collect::<Vec<_>>();
    changed.then(|| format!("{name}[{}]", next.join(",")))
}

fn modded_age(host: &dyn ServerHost, id: &str, state: &str, valid: bool) -> Option<u32> {
    // BonemealableBlock subclasses are not exposed through the WASM host.
    // Only accept registered non-vanilla targets the game deems bonemealable.
    if !valid || id.starts_with("minecraft:") || !state.contains("age=") {
        return None;
    }
    // The source asks the crop to grow. The public API can safely probe
    // registered state values but cannot call arbitrary growCrop methods.
    for age in (0..=31).rev() {
        let Some(next) = next_age(state, age, true) else { continue; };
        if host.block_state_id(&next).is_some() { return Some(age); }
    }
    None
}

fn empty(host: &dyn ServerHost, level: mod_api::LevelRef, pos: [i32; 3]) -> bool {
    level.block(host, pos).is_some_and(|v| matches!(
        v.id(host).as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    ))
}

fn grow_tall(host: &mut dyn ServerHost, level: mod_api::LevelRef,
             pos: [i32; 3], id: &str, cactus: bool) -> bool {
    let mut bottom = pos;
    for _ in 0..2 {
        let Some(y) = bottom[1].checked_sub(1) else { break; };
        let below = [bottom[0], y, bottom[2]];
        if level.block(host, below).is_none_or(|v| v.id(host) != id) { break; }
        bottom = below;
    }
    let mut top = pos;
    for _ in 0..2 {
        let Some(y) = top[1].checked_add(1) else { break; };
        let above = [top[0], y, top[2]];
        if level.block(host, above).is_none_or(|v| v.id(host) != id) { break; }
        top = above;
    }
    if top[1].saturating_sub(bottom[1]) >= 2 { return false; }
    let Some(y) = top[1].checked_add(1) else { return false; };
    let at = [top[0], y, top[2]];
    if !empty(host, level, at) { return false; }
    if cactus {
        for delta in [[-1, 0, 0], [1, 0, 0], [0, 0, -1], [0, 0, 1]] {
            let around = [at[0] + delta[0], at[1], at[2] + delta[2]];
            if !empty(host, level, around) { return false; }
        }
    }
    let Some(state) = host.block_state_id(id) else { return false; };
    level.set_block(host, at, state, flags()) == Ok(true)
}

fn grow_vine(host: &mut dyn ServerHost, level: mod_api::LevelRef,
             pos: [i32; 3], state: mod_api::BlockState) -> bool {
    let Some(y) = pos[1].checked_sub(1) else { return false; };
    let below = [pos[0], y, pos[2]];
    if !empty(host, level, below) { return false; }
    level.set_block(host, below, state, flags()) == Ok(true)
}

pub fn apply(side: HostSide<'_>, event: &mut BonemealEvent<'_>) -> Flow {
    let HostSide::Server(host) = side else { return Flow::Continue; };
    if !on(host, "extended_bonemeal") { return Flow::Continue; }
    let Some(stack) = event.stack.as_ref() else { return Flow::Continue; };
    if stack.item().id(host) != "minecraft:bone_meal" || stack.count() == 0 {
        return Flow::Continue;
    }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if event.player != Some(player.uuid()) || player.may_build(host) != Some(true) {
        return Flow::Continue;
    }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    let pos = [event.pos.x, event.pos.y, event.pos.z];
    let id = event.state.block().id(host);
    let sneaking = player.is_secondary_use_active(host) == Some(true);
    let state = host.block_state_string(event.state);
    let age_max = match id.as_str() {
        "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes" => Some(7),
        "minecraft:beetroots" | "minecraft:nether_wart" | "minecraft:sweet_berry_bush" => Some(3),
        "minecraft:cocoa" => Some(2),
        _ => state.as_deref().and_then(|text| {
            modded_age(host, &id, text, event.valid_bonemeal_target)
        }),
    };
    let grown = if let Some(max) = age_max {
        if id == "minecraft:nether_wart" && !flag(host, "bonemeal_nether_wart", true) {
            false
        } else if id != "minecraft:nether_wart" && (!sneaking || !flag(host, "bonemeal_instant", true)) {
            false
        } else {
            // The source grows nether wart by one age even while crouching.
            let instant = id != "minecraft:nether_wart"
                && sneaking && flag(host, "bonemeal_instant", true);
            state.and_then(|text| next_age(&text, max, instant))
                .and_then(|text| host.block_state_id(&text))
                .is_some_and(|next| level.set_block(host, pos, next, flags()) == Ok(true))
        }
    } else {
        match id.as_str() {
            "minecraft:cactus" if flag(host, "bonemeal_cactus", true) =>
                grow_tall(host, level, pos, &id, true),
            "minecraft:sugar_cane" if flag(host, "bonemeal_sugar_cane", true) =>
                grow_tall(host, level, pos, &id, false),
            "minecraft:vine" if flag(host, "bonemeal_vines", true) =>
                grow_vine(host, level, pos, event.state),
            _ => false,
        }
    };
    if !grown { return Flow::Continue; }
    if player.is_creative(host) != Some(true) {
        let mut spent = stack.clone();
        let left = spent.count() - 1;
        if left == 0 { *event.stack = None; }
        else if spent.set_count(left).is_ok() { *event.stack = Some(spent); }
    }
    event.successful = true;
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn age_limit_and_other_properties_are_preserved() {
        assert_eq!(next_age("minecraft:nether_wart[age=0]", 3, false), Some("minecraft:nether_wart[age=1]".into()));
        assert_eq!(next_age("minecraft:nether_wart[age=2]", 3, false), Some("minecraft:nether_wart[age=3]".into()));
        assert_eq!(next_age("minecraft:wheat[age=2]", 7, true), Some("minecraft:wheat[age=7]".into()));
        assert_eq!(next_age("minecraft:wheat[age=7]", 7, false), None);
        assert_eq!(next_age("minecraft:cocoa[age=1,facing=north]", 2, false), Some("minecraft:cocoa[age=2,facing=north]".into()));
    }
}
