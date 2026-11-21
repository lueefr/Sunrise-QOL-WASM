//! Kelp fertilization uses Sunrise's authoritative vanilla growth helper.
use crate::settings::on;
use mod_api::{Flow, Hand, RightClickBlockEvent, ServerHost};

pub fn click(host: &mut dyn ServerHost, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "kelp_fertilizer") { return Flow::Continue; }
    let Some(held) = event.click.stack else { return Flow::Continue; };
    if held.count() == 0 || held.item().id(host) != "minecraft:kelp" { return Flow::Continue; }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if player.may_build(host) != Some(true) { return Flow::Continue; }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }
    let Some(creative) = player.is_creative(host) else { return Flow::Continue; };
    let slot = match event.click.hand {
        Hand::Off => 40,
        Hand::Main => match player.selected_slot(host) { Some(s) => s as u32, None => return Flow::Continue },
    };
    if !level.bone_meal(host, event.click.pos) { return Flow::Continue; }
    if !creative {
        let mut rest = held.clone();
        let count = rest.count() - 1;
        let remaining = if count == 0 { None } else {
            if rest.set_count(count).is_err() { return Flow::Cancel; }
            Some(rest)
        };
        let _ = player.set_inventory_slot(host, slot, remaining);
    }
    if let Some(living) = host.living(player.entity()) {
        let _ = living.swing(host, event.click.hand, false);
    }
    // Prevent a successful growth action from also placing the kelp.
    Flow::Cancel
}
