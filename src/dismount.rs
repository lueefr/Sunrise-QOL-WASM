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
    if passengers.is_empty() { return None; }
    let allow_players = flag(host, "dismount_players", false);
    for passenger in passengers {
        if !allow_players && passenger.kind(host).is_some_and(|kind| kind.id(host) == "minecraft:player") {
            continue;
        }
        let _ = passenger.stop_riding(host);
    }
    // Upstream consumes the interaction whenever the target had passengers,
    // including when player passengers were protected by configuration.
    Some(InteractionResult::SuccessServer)
}
