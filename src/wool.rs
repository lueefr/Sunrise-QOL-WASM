//! Wool Tweaks interactions through native block writes and inventory
//! authority, preserving both bed halves. Recipes use wool_recipes admission.
use crate::settings::on;
use mod_api::{BlockUpdateFlags, Flow, Hand, RightClickBlockEvent, ServerHost};

const COLORS: &[&str] = &[
    "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
    "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black",
];

fn dye_color(id: &str) -> Option<&str> {
    let color = id.strip_prefix("minecraft:")?.strip_suffix("_dye")?;
    COLORS.contains(&color).then_some(color)
}

fn colored(id: &str, material: &str) -> bool {
    id.strip_prefix("minecraft:").and_then(|p| p.strip_suffix(material))
        .is_some_and(|color| COLORS.contains(&color))
}

fn replacement(color: &str, original: &str, wool_tag: bool) -> Option<String> {
    let material = if wool_tag { "wool" } else if colored(original, "_carpet") { "carpet" }
        else if colored(original, "_bed") { "bed" } else { return None; };
    let next = format!("minecraft:{color}_{material}");
    (next != original).then_some(next)
}

fn property<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    state.split_once('[')?.1.strip_suffix(']')?.split(',').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == key).then_some(value)
    })
}

// The partner and replacement states are resolved before touching the world.
fn bed_pair(state: &str, pos: [i32; 3], next: &str) -> Option<([i32; 3], String, String)> {
    let facing = property(state, "facing")?;
    let (dx, dz): (i32, i32) = match facing {
        "north" => (0, -1), "south" => (0, 1),
        "east" => (1, 0), "west" => (-1, 0), _ => return None,
    };
    let part = property(state, "part")?;
    let (sign, other) = match part {
        "head" => (-1, "foot"), "foot" => (1, "head"), _ => return None,
    };
    let occupied = property(state, "occupied")?;
    if !matches!(occupied, "true" | "false") { return None; }
    let partner = [pos[0].checked_add(dx * sign)?, pos[1], pos[2].checked_add(dz * sign)?];
    let make = |part| format!("{next}[facing={facing},occupied={occupied},part={part}]");
    Some((partner, make(part), make(other)))
}

fn restore(host: &mut dyn ServerHost, level: mod_api::LevelRef,
           originals: &[([i32; 3], mod_api::BlockState)]) {
    // Avoid tearing the other half down while restoring a pair after refusal.
    for (pos, state) in originals {
        let _ = level.set_block(host, *pos, *state,
            BlockUpdateFlags::CLIENTS | BlockUpdateFlags::KNOWN_SHAPE);
    }
}

pub fn click(host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "wool_tweaks") { return Flow::Continue; }
    let Some(held) = event.click.stack else { return Flow::Continue; };
    if held.count() == 0 { return Flow::Continue; }
    let item = held.item().id(host);
    let Some(color) = dye_color(&item) else { return Flow::Continue; };
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }
    let pos = event.click.pos;
    let Some(original) = level.block_state(host, pos) else { return Flow::Continue; };
    let block = original.block();
    let Some(next) = replacement(color, &block.id(host),
        host.block_in_tag(block, "minecraft:wool") == Some(true)) else { return Flow::Continue; };
    let mut originals = vec![(pos, original)];
    let mut writes = Vec::new();
    if colored(&block.id(host), "_bed") {
        let Some(text) = host.block_state_string(original) else { return Flow::Continue; };
        let Some((other, target, partner)) = bed_pair(&text, pos, &next) else { return Flow::Continue; };
        let Some(old_partner) = level.block_state(host, other) else { return Flow::Continue; };
        let Some(partner_text) = host.block_state_string(old_partner) else { return Flow::Continue; };
        if old_partner.block() != block
            || property(&partner_text, "facing") != property(&text, "facing")
            || property(&partner_text, "part") == property(&text, "part") {
            return Flow::Continue;
        }
        let (Some(target), Some(partner), Some(air)) = (
            host.block_state_id(&target), host.block_state_id(&partner), host.block_state_id("minecraft:air"),
        ) else { return Flow::Continue; };
        originals.push((other, old_partner));
        let (head, foot) = if property(&text, "part") == Some("head") { (pos, other) } else { (other, pos) };
        // Upstream clears the head before the foot, then replaces the partner
        // before the clicked half. Preserve that neighbor-update ordering.
        writes.extend([(head, air), (foot, air), (other, partner), (pos, target)]);
    } else {
        let Some(next) = host.block_state_id(&next) else { return Flow::Continue; };
        writes.push((pos, next));
    }
    let Some(creative) = player.is_creative(host) else { return Flow::Continue; };
    let slot = match event.click.hand {
        Hand::Off => 40,
        Hand::Main => {
            let Some(slot) = player.selected_slot(host).and_then(|s| u32::try_from(s).ok()) else {
                return Flow::Continue;
            };
            slot
        }
    };
    let flags = BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS;
    for (at, state) in writes {
        // Clearing the head may already remove the foot through vanilla
        // neighbor updates. That expected no-op is not a refused write.
        if level.block_state(host, at) == Some(state) { continue; }
        if level.set_block(host, at, state, flags) != Ok(true) {
            restore(host, level, &originals);
            return Flow::Continue;
        }
    }
    if !creative {
        let mut rest = held.clone();
        let left = rest.count() - 1;
        let remaining = if left == 0 { None } else {
            if rest.set_count(left).is_err() {
                restore(host, level, &originals);
                return Flow::Continue;
            }
            Some(rest)
        };
        if player.set_inventory_slot(host, slot, remaining).is_err() {
            restore(host, level, &originals);
            return Flow::Continue;
        }
    }
    if let Some(living) = host.living(player.entity()) {
        let _ = living.swing(host, event.click.hand, false);
    }
    // The original callback returns PASS even after dyeing, so preserve the
    // normal interaction instead of claiming a vanilla action was cancelled.
    Flow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_vanilla_dyes_are_recognized_without_suffix_false_positives() {
        for color in COLORS {
            assert_eq!(dye_color(&format!("minecraft:{color}_dye")), Some(*color));
        }
        assert_eq!(dye_color("example:red_dye"), None);
        assert_eq!(dye_color("minecraft:rainbow_dye"), None);
        assert_eq!(dye_color("minecraft:red_wool"), None);
    }
    #[test]
    fn dye_changes_only_supported_materials_and_spends_nothing_on_same_color() {
        assert_eq!(replacement("red", "minecraft:white_wool", true).as_deref(), Some("minecraft:red_wool"));
        assert_eq!(replacement("red", "mod:custom_wool", true).as_deref(), Some("minecraft:red_wool"));
        assert_eq!(replacement("blue", "minecraft:red_carpet", false).as_deref(), Some("minecraft:blue_carpet"));
        assert_eq!(replacement("red", "minecraft:red_wool", true), None);
        assert_eq!(replacement("red", "minecraft:red_carpet", false), None);
        assert_eq!(replacement("red", "minecraft:moss_carpet", false), None);
        assert_eq!(replacement("red", "minecraft:red_bed", false), None);
        assert_eq!(replacement("red", "minecraft:white_bed", false).as_deref(), Some("minecraft:red_bed"));
        assert_eq!(replacement("red", "minecraft:stone", false), None);
    }
    #[test]
    fn bed_partners_match_each_direction_and_clicked_half() {
        for (facing, direction) in [("north", [0, -1]), ("south", [0, 1]), ("east", [1, 0]), ("west", [-1, 0])] {
            for (part, sign, other) in [("head", -1, "foot"), ("foot", 1, "head")] {
                for occupied in ["true", "false"] {
                    let state = format!("minecraft:white_bed[facing={facing},occupied={occupied},part={part}]");
                    let (partner, target, next_partner) = bed_pair(&state, [5, 70, -8], "minecraft:red_bed").unwrap();
                    assert_eq!(partner, [5 + direction[0] * sign, 70, -8 + direction[1] * sign]);
                    assert_eq!(property(&target, "part"), Some(part));
                    assert_eq!(property(&next_partner, "part"), Some(other));
                    assert_eq!(property(&target, "occupied"), Some(occupied));
                    assert_eq!(property(&next_partner, "occupied"), Some(occupied));
                    assert_eq!(property(&target, "facing"), Some(facing));
                }
            }
        }
    }
    #[test]
    fn malformed_or_overflowing_bed_state_is_rejected() {
        assert!(bed_pair("bed[part=foot,facing=north,occupied=false]", [0, 0, i32::MIN], "bed").is_none());
        assert!(bed_pair("bed[part=foot,facing=up,occupied=false]", [0; 3], "bed").is_none());
        assert!(bed_pair("bed[part=foot,facing=north]", [0; 3], "bed").is_none());
    }

}
