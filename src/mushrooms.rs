//! Surface Mushrooms: bypass the placement light limit on solid surfaces.
use crate::settings::on;
use mod_api::{HostBlockStates, BlockUpdateFlags, Flow, Hand, RightClickBlockEvent, ServerHost};

pub fn click(host: &mut dyn ServerHost, event: &RightClickBlockEvent<'_>) -> Flow {
    if !on(host, "surface_mushrooms") { return Flow::Continue; }
    let Some(stack) = event.click.stack else { return Flow::Continue; };
    let id = stack.item().id(host);
    if !matches!(id.as_str(), "minecraft:red_mushroom" | "minecraft:brown_mushroom") { return Flow::Continue; }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    if player.may_build(host) != Some(true) { return Flow::Continue; }
    let Some(level) = player.level(host) else { return Flow::Continue; };
    let Some(state) = level.block_state(host, event.click.pos) else { return Flow::Continue; };
    if !level.in_place(host) || host.block_states().is_solid_render(state) != Some(true) { return Flow::Continue; }
    let [x, y, z] = event.click.pos;
    let Some(y) = y.checked_add(1) else { return Flow::Continue; };
    let pos = [x, y, z];
    let Some(previous) = level.block_state(host, pos) else { return Flow::Continue; };
    if previous.block().id(host) != "minecraft:air" { return Flow::Continue; }
    let Some(next) = host.block_state_id(&id) else { return Flow::Continue; };
    let slot = match event.click.hand { Hand::Off => 40, Hand::Main => match player.selected_slot(host) {
        Some(s) => s as u32, None => return Flow::Continue,
    }};
    let Some(creative) = player.is_creative(host) else { return Flow::Continue; };
    let flags = BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS;
    if level.set_block(host, pos, next, flags) != Ok(true) { return Flow::Continue; }
    if !creative {
        let mut rest = stack.clone();
        let count = rest.count() - 1;
        let remaining = if count == 0 { None } else {
            if rest.set_count(count).is_err() { return Flow::Cancel; }
            Some(rest)
        };
        if player.set_inventory_slot(host, slot, remaining).is_err() {
            let _ = level.set_block(host, pos, previous, flags);
            return Flow::Cancel;
        }
    }
    if let Some(living) = host.living(player.entity()) {
        let _ = living.swing(host, event.click.hand, false);
    }
    Flow::Cancel
}
