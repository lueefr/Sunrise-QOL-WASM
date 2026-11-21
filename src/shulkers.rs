//! Shulkers are queued only after confirmed deaths, never after canceled deaths.
//! Each tick handles a bounded number of spawns; no level is loaded implicitly.
use crate::settings::{amount, flag, on};
use mod_api::{Compound, LivingDeathEvent, LivingDeathPostEvent, ServerHost, ServerTickEvent, ServerTickPhase, SpawnReason, Tag};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_PENDING: usize = 1024;
const PER_TICK: usize = 32;
const MAX_RETRIES: u8 = 30;

#[derive(Clone)]
struct Death {
    id: u32,
    dimension: String,
    position: [f64; 3],
    nbt: Option<Compound>,
    time: u64,
}

#[derive(Clone)]
struct Respawn {
    dimension: String,
    position: [f64; 3],
    nbt: Option<Compound>,
    due: u64,
    retries: u8,
}

#[derive(Default)]
pub struct Shulkers {
    time: AtomicU64,
    deaths: Mutex<VecDeque<Death>>,
    queue: Mutex<VecDeque<Respawn>>,
}

impl Shulkers {
    pub fn death(&self, host: &mut dyn ServerHost, event: &LivingDeathEvent<'_>) {
        if event.kind != "minecraft:shulker" || !on(host, "respawning_shulkers") { return; }
        let Some(entity) = host.entity(event.entity) else { return; };
        if let Some(tags) = entity.tags(host) {
            if tags.iter().any(|tag| tag == "flowermimics.mimic") { return; }
            if flag(host, "skip_spawner", true)
                && tags.iter().any(|tag| tag == "collective.fromspawner") { return; }
        }
        let (Some(level), Some(position)) = (entity.level(host), entity.position(host)) else { return; };
        let id = event.entity;
        let Some(dimension) = level.dimension().id(host) else { return; };
        let mut deaths = match self.deaths.lock() { Ok(guard) => guard, Err(_) => return };
        if deaths.len() >= MAX_PENDING { deaths.pop_front(); }
        // The upstream restores the dead shulker's data onto the new entity,
        // rather than spawning a fresh shulker with default color/name/state.
        let nbt = entity.nbt(host);
        deaths.push_back(Death {
            id, dimension, position, nbt,
            time: self.time.load(Ordering::Relaxed),
        });
    }

    pub fn confirmed(&self, host: &dyn ServerHost, event: &LivingDeathPostEvent<'_>) {
        if !on(host, "respawning_shulkers") { return; }
        let death = self.deaths.lock().ok().and_then(|mut deaths| {
            let index = deaths.iter().position(|entry| entry.id == event.entity)?;
            deaths.remove(index)
        });
        let Some(death) = death else { return; };
        let delay = amount(host, "respawn_ticks", 1200., 1., 72000.) as u64;
        if let Ok(mut queue) = self.queue.lock() {
            if queue.len() >= MAX_PENDING { queue.pop_front(); }
            queue.push_back(Respawn {
                dimension: death.dimension, position: death.position, nbt: death.nbt,
                due: self.time.load(Ordering::Relaxed).saturating_add(delay), retries: 0,
            });
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let now = self.time.fetch_add(1, Ordering::Relaxed) + 1;
        if !on(host, "respawning_shulkers") {
            self.reset();
            return;
        }
        if let Ok(mut deaths) = self.deaths.lock() {
            deaths.retain(|death| now.saturating_sub(death.time) < 100);
        }
        for _ in 0..PER_TICK {
            let task = self.queue.lock().ok().and_then(|mut queue| {
                let index = queue.iter().position(|entry| entry.due <= now)?;
                queue.remove(index)
            });
            let Some(mut task) = task else { break; };
            let spawned = host.level(&task.dimension).is_some_and(|level| {
                // Preserve color, custom name and other persistent shulker data,
                // but never reuse the dead entity's UUID or zero health.
                let restored = task.nbt.clone().and_then(|mut nbt| {
                    for key in ["UUID", "UUIDMost", "UUIDLeast", "DeathTime", "HurtTime"] {
                        nbt.remove(&key.into());
                    }
                    nbt.insert("Health".into(), Tag::Float(30_f32.to_bits()));
                    host.summon(&task.dimension, "minecraft:shulker", task.position, nbt)
                });
                restored.is_some() || level.spawn_mob(host, "minecraft:shulker",
                    task.position, SpawnReason::Natural).is_some()
            });
            if !spawned && task.retries < MAX_RETRIES {
                task.retries += 1;
                task.due = now.saturating_add(20 * 10);
                if let Ok(mut queue) = self.queue.lock() { queue.push_back(task); }
            }
        }
    }

    pub fn reset(&self) {
        self.time.store(0, Ordering::Relaxed);
        if let Ok(mut deaths) = self.deaths.lock() { deaths.clear(); }
        if let Ok(mut queue) = self.queue.lock() { queue.clear(); }
    }
}
