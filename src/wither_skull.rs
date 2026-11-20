//! Always a Wither Skull follows the original delayed post-death inspection.
//! The original checks live item entities one server tick after the skeleton drops loot.
use crate::settings::on;
use mod_api::{
    Identifier, ItemStack, LivingDeathEvent, LivingDeathPostEvent, ServerHost,
    ServerTickEvent, ServerTickPhase,
};
use std::collections::VecDeque;
use std::sync::{Mutex, atomic::{AtomicU64, Ordering}};

const CAP: usize = 1024;
const PER_TICK: usize = 64;

#[derive(Clone)]
struct Death {
    id: u32,
    dimension: String,
    pos: [f64; 3],
    time: u64,
}

#[derive(Clone)]
struct Pending {
    dimension: String,
    pos: [f64; 3],
    due: u64,
}

#[derive(Default)]
pub struct WitherSkulls {
    time: AtomicU64,
    deaths: Mutex<VecDeque<Death>>,
    pending: Mutex<VecDeque<Pending>>,
}

fn bounds(pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    ([pos[0] - 1., pos[1] - 1., pos[2] - 1.],
     [pos[0] + 1., pos[1] + 1., pos[2] + 1.])
}

fn fresh(age: i32, kind: &str) -> bool {
    kind == "minecraft:wither_skeleton_skull" && age < 10
}

impl WitherSkulls {
    pub fn dying(&self, host: &dyn ServerHost, event: &LivingDeathEvent<'_>) {
        if !on(host, "always_wither_skull") || event.kind != "minecraft:wither_skeleton" {
            return;
        }
        let Some(entity) = host.entity(event.entity) else { return; };
        let (Some(level), Some(pos)) = (entity.level(host), entity.position(host)) else { return; };
        let Some(dimension) = level.dimension().id(host) else { return; };
        if let Ok(mut deaths) = self.deaths.lock() {
            if deaths.len() >= CAP { deaths.pop_front(); }
            deaths.push_back(Death {
                id: event.entity,
                dimension, pos, time: self.time.load(Ordering::Relaxed),
            });
        }
    }

    pub fn confirmed(&self, host: &dyn ServerHost, event: &LivingDeathPostEvent<'_>) {
        let death = self.deaths.lock().ok().and_then(|mut deaths| {
            let index = deaths.iter().position(|death| death.id == event.entity)?;
            deaths.remove(index)
        });
        let Some(death) = death else { return; };
        if !on(host, "always_wither_skull") { return; }
        let due = self.time.load(Ordering::Relaxed).saturating_add(1);
        if let Ok(mut pending) = self.pending.lock() {
            if pending.len() >= CAP { pending.pop_front(); }
            pending.push_back(Pending { dimension: death.dimension, pos: death.pos, due });
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let now = self.time.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        if !on(host, "always_wither_skull") {
            self.reset();
            return;
        }
        if let Ok(mut deaths) = self.deaths.lock() {
            deaths.retain(|death| now.saturating_sub(death.time) < 100);
        }
        for _ in 0..PER_TICK {
            let task = self.pending.lock().ok().and_then(|mut pending| {
                let index = pending.iter().position(|task| task.due <= now)?;
                pending.remove(index)
            });
            let Some(task) = task else { break; };
            let Some(level) = host.level(&task.dimension) else { continue; };
            let (min, max) = bounds(task.pos);
            let Some(entities) = level.entities_in(host, min, max) else { continue; };
            let found = entities.iter().any(|entity| {
                host.item_entity(*entity).and_then(|item| {
                    let stack = item.stack(host)?;
                    let age = item.age(host)?;
                    Some(fresh(age, &stack.item().id(host)))
                }) == Some(true)
            });
            if found { continue; }
            let Some(id) = Identifier::parse("minecraft:wither_skeleton_skull").ok() else {
                continue;
            };
            let Some(item) = host.item_key(&id) else { continue; };
            let Some(stack) = ItemStack::new(item, 1) else { continue; };
            let _ = level.spawn_item(host, task.pos[0], task.pos[1], task.pos[2], stack, 0);
        }
    }

    pub fn reset(&self) {
        self.time.store(0, Ordering::Relaxed);
        if let Ok(mut deaths) = self.deaths.lock() { deaths.clear(); }
        if let Ok(mut pending) = self.pending.lock() { pending.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn follows_original_one_block_and_fresh_item_rule() {
        assert_eq!(bounds([10., 64., -5.]),
                   ([9., 63., -6.], [11., 65., -4.]));
        assert!(fresh(0, "minecraft:wither_skeleton_skull"));
        assert!(fresh(9, "minecraft:wither_skeleton_skull"));
        assert!(!fresh(10, "minecraft:wither_skeleton_skull"));
        assert!(!fresh(0, "minecraft:coal"));
    }
}
