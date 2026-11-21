//! Initial Wool Tweaks interaction port. Crafting recipes and bed recoloring
//! remain explicitly pending; use native block writes and inventory authority.
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

fn carpet(id: &str) -> bool {
    id.strip_prefix("minecraft:").and_then(|p| p.strip_suffix("_carpet"))
        .is_some_and(|color| COLORS.contains(&color))
}

fn replacement(color: &str, original: &str, wool_tag: bool) -> Option<String> {
    let material = if wool_tag { "wool" } else if carpet(original) { "carpet" } else { return None; };
    let next = format!("minecraft:{color}_{material}");
    (next != original).then_some(next)
}

pub fn click(host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "wool_tweaks") { return Flow::Continue; }
    let Some(held) = event.click.stack else { return Flow::Continue; };
    if held.count() == 0 { return Flow::Continue; }
    let item = held.item().id(host);
    let Some(color) = dye_color(&item) else { return Flow::Continue; };
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if player.is_spectator(host) != Some(false) || player.may_build(host) != Some(true) {
        return Flow::Continue;
    }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }
    let pos = event.click.pos;
    let Some(original) = level.block_state(host, pos) else { return Flow::Continue; };
    let block = original.block();
    let Some(next) = replacement(color, &block.id(host),
        host.block_in_tag(block, "minecraft:wool") == Some(true)) else { return Flow::Continue; };
    let Some(next) = host.block_state_id(&next) else { return Flow::Continue; };
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
    if level.set_block(host, pos, next, flags) != Ok(true) { return Flow::Continue; }
    if !creative {
        let mut rest = held.clone();
        let left = rest.count() - 1;
        let remaining = if left == 0 { None } else {
            if rest.set_count(left).is_err() {
                let _ = level.set_block(host, pos, original, flags);
                return Flow::Continue;
            }
            Some(rest)
        };
        if player.set_inventory_slot(host, slot, remaining).is_err() {
            let _ = level.set_block(host, pos, original, flags);
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
        assert_eq!(replacement("red", "minecraft:stone", false), None);
    }
}
