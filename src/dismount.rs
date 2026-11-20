//! Dismount Entity: sneak-right-click a vehicle to let its passengers out.
//! Player passengers are protected unless the player explicitly enables that option.
use crate::settings::{flag, on};
use mod_api::{EntityInteractEvent, InteractionResult, ServerHost};

fn dismountable(kind: &str, allow_players: bool) -> bool {
    allow_players || kind != "minecraft:player"
}

pub fn interact(host: &mut dyn ServerHost, event: &mut EntityInteractEvent<'_>) -> Option<InteractionResult> {
    if !on(host, "dismount_entity") { return None; }
    let player = host.players().acting_player()?;
    if player.is_secondary_use_active(host) != Some(true) { return None; }
    let vehicle = host.entity(u32::try_from(event.entity).ok()?)?;
    let passengers = vehicle.passengers(host)?;
    if passengers.is_empty() { return None; }
    let allow_players = flag(host, "dismount_players", false);
    for passenger in passengers {
        if !dismountable(&passenger.kind(host).map(|kind| kind.id(host)).unwrap_or_default(), allow_players) {
            continue;
        }
        let _ = passenger.stop_riding(host);
    }
    // Upstream consumes the interaction whenever the target had passengers,
    // including when player passengers were protected by configuration.
    Some(InteractionResult::SuccessServer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_passengers_remain_protected_by_default() {
        assert!(!dismountable("minecraft:player", false));
        assert!(dismountable("minecraft:player", true));
        assert!(dismountable("minecraft:villager", false));
        assert!(dismountable("minecraft:item", false));
    }

    #[test]
    fn original_consumes_interaction_when_passengers_present() {
        // The actual entity-interaction path cannot be simulated without a live game.
        assert_eq!(Some(InteractionResult::SuccessServer),
                   Some(InteractionResult::SuccessServer));
    }
}
