//! Softer Hay Bales: the original suppresses fall damage only for players
//! landing on hay. A block part changes the authoritative landing multiplier.
use crate::settings::on;
use mod_api::{BlockPart, FallOnEvent, LevelRandomHost, Registrar, ServerHost};
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

fn protected(kind: &str, enabled: bool) -> bool {
    enabled && kind == "minecraft:player"
}

struct SoftHay;

#[mod_api::sunrise_part]
impl BlockPart for SoftHay {
    fn fall_on(&self, _: &mut dyn LevelRandomHost, event: &mut FallOnEvent<'_>) {
        if protected(event.kind, ENABLED.load(Ordering::Relaxed)) {
            // The multiplier is zero, preserving vanilla landing block effects
            // without adding an unrelated fall-damage cancel hook.
            event.damage_multiplier = 0.;
        }
    }
}

pub fn register(registrar: &mut dyn Registrar) {
    let key = registrar.block_key("minecraft:hay_block");
    registrar.block_part(key, SoftHay);
}

pub fn refresh(host: &dyn ServerHost) {
    ENABLED.store(on(host, "softer_hay_bales"), Ordering::Relaxed);
}

pub fn reset() {
    ENABLED.store(false, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_players_on_enabled_hay_are_immune() {
        assert!(protected("minecraft:player", true));
        assert!(!protected("minecraft:player", false));
        assert!(!protected("minecraft:zombie", true));
        assert!(!protected("minecraft:item", true));
    }
}
