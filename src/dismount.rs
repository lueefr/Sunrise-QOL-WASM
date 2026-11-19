//! Dismount Entity: sneak-right-click a vehicle to let its passengers out.
//! Player passengers are protected unless the player explicitly enables that option.
use crate::settings::{flag, on};
use mod_api::{EntityInteractEvent, InteractionResult, ServerHost};

pub fn interact(host: &mut dyn ServerHost, event: &mut EntityInteractEvent<'_>) -> Option<InteractionResult> {
    if !on(host, "dismount_entity") { return None; }
    let player = host.players().acting_player()?;
    if player.is_secondary_use_active(host) != Some(true) { return None; }
    let vehicle = host.entity(u32::try_from(event.entity).ok()?)?;
    let passengers = vehicle.passengers(host)?;
    let allow_players = flag(host, "dismount_players", false);
    let mut changed = false;
    for passenger in passengers.into_iter().take(8) {
        if !allow_players && passenger.kind(host).is_some_and(|kind| kind.id(host) == "minecraft:player") {
            continue;
        }
        if passenger.stop_riding(host) { changed = true; }
    }
    changed.then_some(InteractionResult::SuccessServer)
}
