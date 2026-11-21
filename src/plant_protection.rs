//! Snapshot equipment/effects once per server tick for restricted hit/trample hosts.
use crate::settings::on;
use mod_api::{ComponentId, EquipmentSlot, Flow, ItemResource, JavaString,
    LivingIncomingDamageEvent, FarmlandTrampleEvent, ServerHost, ServerTickEvent, ServerTickPhase, Tag};
use std::collections::BTreeSet;
use std::sync::Mutex;

#[derive(Default)]
struct Snapshot { berry: BTreeSet<i32>, feather: BTreeSet<u32> }
#[derive(Default)]
pub struct Protection { state: Mutex<Snapshot> }

fn feather_falling(host: &dyn ServerHost, stack: &mod_api::ItemStack) -> bool {
    let Some(id) = ComponentId::new("minecraft:enchantments") else { return false; };
    let Some(Tag::Compound(values)) = ItemResource::of(stack).value(host, &id) else { return false; };
    match values.get(&JavaString::from("minecraft:feather_falling")) {
        Some(Tag::Int(n)) => *n > 0,
        Some(Tag::Short(n)) => *n > 0,
        Some(Tag::Byte(n)) => *n > 0,
        _ => false,
    }
}

impl Protection {
    pub fn tick(&self, host: &dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) { return; }
        let berry = on(host, "thorny_bush_protection");
        let feather = on(host, "no_feather_trample");
        let mut next = Snapshot::default();
        if berry || feather {
            for player in host.players().server_players() {
                let Some(living) = host.living(player.entity()) else { continue; };
                if berry && living.equipment(host, EquipmentSlot::Legs).is_some_and(|s| s.count() > 0) {
                    next.berry.insert(player.entity().id() as i32);
                }
                if feather && (living.equipment(host, EquipmentSlot::Feet).is_some_and(|s| feather_falling(host, &s))
                    || host.effect_key("minecraft:slow_falling").is_some_and(|key| living.effect(host, key).is_some())) {
                    next.feather.insert(player.entity().id() as u32);
                }
            }
        }
        if let Ok(mut state) = self.state.lock() { *state = next; }
    }
    pub fn damage(&self, event: &LivingIncomingDamageEvent<'_>) -> Flow {
        if event.kind == "minecraft:player" && event.source.damage_type == "minecraft:sweet_berry_bush"
            && self.state.lock().is_ok_and(|s| s.berry.contains(&event.entity)) {
            return Flow::Cancel;
        }
        Flow::Continue
    }
    pub fn trample(&self, event: &FarmlandTrampleEvent) -> Flow {
        if self.state.lock().is_ok_and(|s| s.feather.contains(&event.entity)) { Flow::Cancel }
        else { Flow::Continue }
    }
    pub fn reset(&self) { if let Ok(mut state) = self.state.lock() { *state = Snapshot::default(); } }
}
