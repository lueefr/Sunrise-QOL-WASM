//! Followers Teleport Too: capture owned, standing pets at command source and
//! move those exact entities only after the owner's command teleport succeeds.
use crate::settings::{amount, on};
use mod_api::{
    EntityRef, Flow, LeashHolder, PlayerTeleportedEvent, ServerHost, Tag, TeleportCommandEvent,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

const MAX_PETS: usize = 32;
const MAX_PENDING: usize = 64;

#[derive(Clone)]
struct Pending {
    from: [f64; 3],
    target: String,
    pets: Vec<EntityRef>,
}

#[derive(Default)]
pub struct Followers {
    pending: Mutex<BTreeMap<u128, Pending>>,
}

fn owner(nbt: &mod_api::Compound, player: u128) -> bool {
    match nbt.get(&"Owner".into()) {
        Some(Tag::IntArray(parts)) if parts.len() == 4 => {
            let mut bytes = [0_u8; 16];
            for (index, part) in parts.iter().enumerate() {
                bytes[index * 4..index * 4 + 4].copy_from_slice(&part.to_be_bytes());
            }
            u128::from_be_bytes(bytes) == player
        }
        Some(Tag::String(value)) => value.to_string().ok().is_some_and(|text| {
            let hex = text.replace('-', "");
            u128::from_str_radix(&hex, 16).ok() == Some(player)
        }),
        _ => false,
    }
}

fn sitting(nbt: &mod_api::Compound) -> bool {
    for field in ["Sitting", "isSitting", "OrderedToSit"] {
        match nbt.get(&field.into()) {
            Some(Tag::Byte(value)) if *value != 0 => return true,
            Some(Tag::Int(value)) if *value != 0 => return true,
            _ => {}
        }
    }
    false
}

fn tamed(id: &str) -> bool {
    matches!(id,
        "minecraft:wolf" | "minecraft:cat" | "minecraft:parrot" |
        "minecraft:horse" | "minecraft:donkey" | "minecraft:mule" | "minecraft:llama"
    )
}

fn close(a: [f64; 3], b: [f64; 3], radius: f64) -> bool {
    a.iter().zip(b).all(|(x, y)| x.is_finite() && y.is_finite()) &&
        a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f64>() <= radius * radius
}

impl Followers {
    pub fn before(&self, host: &mut dyn ServerHost, event: &mut TeleportCommandEvent<'_>) -> Flow {
        if !on(host, "followers_teleport") { return Flow::Continue; }
        let Some(player) = host.players().server_players().into_iter()
            .find(|p| p.entity().id() == event.entity) else { return Flow::Continue; };
        if player.is_spectator(host) != Some(false) { return Flow::Continue; }
        let Some(level) = player.level(host) else { return Flow::Continue; };
        let uuid = player.uuid();
        let radius = amount(host, "followers_radius", 50., 1., 256.);
        let from = event.from;
        let min = from.map(|v| v - radius);
        let max = from.map(|v| v + radius);
        let Some(entities) = level.entities_in(host, min, max) else { return Flow::Continue; };
        let mut pets = Vec::new();
        for entity in entities.into_iter().take(512) {
            if pets.len() >= MAX_PETS { break; }
            let Some(kind) = entity.kind(host) else { continue; };
            if !tamed(&kind.id(host)) || !close(from, entity.position(host).unwrap_or([f64::NAN; 3]), radius) {
                continue;
            }
            if entity.leash_holder(host) != Some(LeashHolder::Unleashed) { continue; }
            let Some(nbt) = entity.nbt(host) else { continue; };
            if owner(&nbt, uuid) && !sitting(&nbt) {
                pets.push(entity);
            }
        }
        if let Ok(mut pending) = self.pending.lock() {
            if !pending.contains_key(&uuid) && pending.len() >= MAX_PENDING { pending.clear(); }
            pending.insert(uuid, Pending {
                from, target: event.dimension.to_string(), pets
            });
        }
        Flow::Continue
    }

    pub fn after(&self, host: &mut dyn ServerHost, event: &mut PlayerTeleportedEvent<'_>) {
        if !on(host, "followers_teleport") { return; }
        let pending = self.pending.lock().ok().and_then(|mut map| map.remove(&event.player));
        let Some(pending) = pending else { return; };
        if pending.target != event.dimension || !close(pending.from, event.from, 1.0) { return; }
        let radius = amount(host, "followers_radius", 50., 1., 256.);
        if radius <= 0. { return; }
        // The server owns each move; never teleport arbitrary mobs or parked pets.
        for (index, pet) in pending.pets.iter().take(MAX_PETS).enumerate() {
            let x = ((index % 5) as f64 - 2.) * 1.5;
            let z = ((index / 5) as f64) * 1.5 + 2.;
            let to = [event.to[0] + x, event.to[1], event.to[2] + z];
            let _ = pet.teleport_to_level(host, &pending.target, to, None);
        }
    }

    pub fn reset(&self) {
        if let Ok(mut map) = self.pending.lock() { map.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owners_and_ranges_match_exactly() {
        assert!(close([0., 64., 0.], [30., 64., 40.], 50.));
        assert!(!close([0., 64., 0.], [51., 64., 0.], 50.));
        assert!(tamed("minecraft:wolf"));
        assert!(!tamed("minecraft:zombie"));
    }
}
