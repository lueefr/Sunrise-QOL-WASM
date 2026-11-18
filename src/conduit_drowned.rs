//! Conduit Power suppresses drowned spawning within a horizontal radius.
//! Follows the original range check: only X/Z distance, with no world mutations.
use crate::settings::{amount, on};
use mod_api::{Host, MobSpawnPositionCheckEvent, SpawnCheck};

fn near(first: [f64; 3], other: [f64; 3], radius: f64) -> bool {
    if !first.iter().all(|v| v.is_finite()) || !other.iter().all(|v| v.is_finite()) {
        return false;
    }
    let dx = first[0] - other[0];
    let dz = first[2] - other[2];
    dx * dx + dz * dz < radius * radius
}

pub fn check(host: &dyn Host, event: &mut MobSpawnPositionCheckEvent<'_>) {
    if !on(host, "drowned_prevention") || event.kind != "minecraft:drowned" {
        return;
    }
    let radius = amount(host, "drowned_radius", 400., 0., 400.);
    if radius <= 0. { return; }
    let Some(players) = event.level.online_players(host) else { return; };
    for player in players {
        let Some(pos) = player.entity().position(host) else { continue; };
        if !near(pos, event.pos, radius) { continue; }
        let Some(living) = host.living(player.entity()) else { continue; };
        if living.effects(host).is_some_and(|effects| {
            effects.iter().any(|effect| effect.id.id(host) == "minecraft:conduit_power")
        }) {
            *event.result = SpawnCheck::Fail;
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizontal_distance_matches_reference() {
        assert!(near([0., 64., 0.], [0., -10., 0.], 400.));
        assert!(!near([400., 64., 0.], [0., 64., 0.], 400.));
        assert!(near([399., 64., 0.], [0., 64., 0.], 400.));
        assert!(!near([f64::NAN, 64., 0.], [0., 64., 0.], 400.));
    }
}
