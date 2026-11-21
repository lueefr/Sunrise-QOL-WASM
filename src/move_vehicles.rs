//! Carry the existing boat/cart entity, preserving cargo and passengers.
use crate::settings::on;
use mod_api::{EntityInteractEvent, Hand, InteractionResult, ServerHost, ServerTickEvent, ServerTickPhase, Flow};
use std::sync::Mutex;

struct Carried { owner: u128, entity: u32, uuid: u128, stood: bool, boat: bool }
#[derive(Default)]
pub struct Vehicles { carried: Mutex<Vec<Carried>> }
fn enabled(host: &dyn ServerHost, boat: bool) -> bool {
    on(host, if boat { "move_boats" } else { "move_minecarts" })
}
fn vehicle(kind: &str) -> Option<bool> {
    let name = kind.strip_prefix("minecraft:")?;
    if name.ends_with("_boat") || name.ends_with("_raft") { Some(true) }
    else if name == "minecart" || name.ends_with("_minecart") { Some(false) } else { None }
}
impl Vehicles {
    pub fn command(&self, host: &mut dyn ServerHost, uuid: u128, command: &str) -> Flow {
        let args: Vec<_> = command.trim_start_matches('/').split_whitespace().collect();
        if args.first() != Some(&"qolcarry") { return Flow::Continue; }
        let Some(player) = host.player(uuid) else { return Flow::Cancel; };
        if args.get(1) == Some(&"drop") {
            if let Ok(mut carried) = self.carried.lock() { carried.retain(|c| c.owner != player.uuid()); }
            return Flow::Cancel;
        }
        let boat = match args.get(1) { Some(&"boat") => true, Some(&"minecart") => false, _ => return Flow::Cancel };
        if !enabled(host, boat) { return Flow::Cancel; }
        let (Some(level), Some(pos)) = (player.level(host), player.entity().position(host)) else { return Flow::Cancel; };
        let entities = level.entities_in(host, pos.map(|n| n - 4.), pos.map(|n| n + 4.)).unwrap_or_default();
        let target = entities.into_iter().filter(|e| e.kind(host).is_some_and(|k| vehicle(&k.id(host)) == Some(boat)))
            .filter_map(|e| e.position(host).map(|at| (e, (0..3).map(|i| (at[i]-pos[i]).powi(2)).sum::<f64>())))
            .min_by(|a,b| a.1.total_cmp(&b.1)).map(|(e,_)| e);
        let Some(entity) = target else { return Flow::Cancel; };
        let Some(uuid) = entity.uuid(host) else { return Flow::Cancel; };
        if let Ok(mut carried) = self.carried.lock() {
            carried.retain(|c| c.owner != player.uuid() && c.uuid != uuid);
            carried.push(Carried { owner: player.uuid(), entity: entity.id(), uuid, stood: true, boat });
        }
        Flow::Cancel
    }
    pub fn click(&self, host: &mut dyn ServerHost, event: &EntityInteractEvent<'_>) -> Option<InteractionResult> {
        let boat = vehicle(event.kind)?;
        if event.hand != Hand::Main || !enabled(host, boat) { return None; }
        let player = host.players().acting_player()?;
        if player.is_secondary_use_active(host) != Some(true) { return None; }
        let entity = host.entity(u32::try_from(event.entity).ok()?)?;
        let uuid = entity.uuid(host)?;
        let mut carried = self.carried.lock().ok()?;
        // One carrier per entity; use UUIDs so recycled entity IDs cannot move another mob.
        carried.retain(|c| c.owner != player.uuid() && c.uuid != uuid);
        carried.push(Carried { owner: player.uuid(), entity: entity.id(), uuid, stood: false, boat });
        Some(InteractionResult::SuccessServer)
    }
    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) { return; }
        let Ok(mut carried) = self.carried.lock() else { return; };
        carried.retain_mut(|carry| {
            if !enabled(host, carry.boat) { return false; }
            let Some(player) = host.player(carry.owner) else { return false; };
            let Some(entity) = host.entity(carry.entity) else { return false; };
            if entity.uuid(host) != Some(carry.uuid) || entity.level(host) != player.level(host)
                || host.living(player.entity()).is_none_or(|l| l.alive(host) != Some(true)) { return false; }
            if entity.passengers(host).is_some_and(|ps| ps.contains(&player.entity())) { return false; }
            let crouch = player.is_secondary_use_active(host) == Some(true);
            if carry.stood && crouch { return false; }
            if !crouch { carry.stood = true; }
            let (Some(pos), Some(eye), Some(look)) = (player.entity().position(host), player.entity().eye_height(host), player.entity().view_vector(host)) else { return false; };
            entity.teleport_to(host, [pos[0] + look[0] * 2., pos[1] + eye, pos[2] + look[2] * 2.], None)
        });
    }
    pub fn reset(&self) { if let Ok(mut carried) = self.carried.lock() { carried.clear(); } }
}
