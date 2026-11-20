//! Dragon Drops Elytra: Sunrise's dragon follows its own 200-tick death path.
//! Track dragon residency and use the vanilla entity-die game event rather than mob loot.
use crate::settings::on;
use mod_api::{
    EntityJoinLevelEvent, EntityLeaveLevelEvent, Identifier, ItemStack, ServerHost,
    ServerTickEvent, ServerTickPhase, Text, VanillaGameEventEvent,
};
use mod_api::text::TextContent;
use std::collections::VecDeque;
use std::sync::{Mutex, atomic::{AtomicU64, Ordering}};

const TRACK_CAP: usize = 64;
const REMOVE_GRACE: u64 = 2;

#[derive(Clone)]
struct Dragon {
    id: u32,
    dimension: String,
    left: Option<u64>,
}

#[derive(Default)]
pub struct DragonDrops {
    tick: AtomicU64,
    dragons: Mutex<VecDeque<Dragon>>,
}

fn death_event(kind: &str, entity: Option<u32>, tracked: u32) -> bool {
    kind == "minecraft:entity_die" && entity == Some(tracked)
}

fn inside(player: [f64; 3], death: [f64; 3]) -> bool {
    player.iter().zip(death).all(|(a, b)| (*a - b).abs() <= 30.)
}

fn translate(key: &str, fallback: &str, color: &str) -> Text {
    let mut text = Text::translate(key, Vec::new());
    if let TextContent::Translatable { fallback: value, .. } = &mut text.content {
        *value = Some(fallback.to_owned());
    }
    text.style.color = Some(color.to_owned());
    text
}

impl DragonDrops {
    pub fn joined(&self, event: &EntityJoinLevelEvent<'_>) {
        if event.kind != "minecraft:ender_dragon" { return; }
        if let Ok(mut dragons) = self.dragons.lock() {
            dragons.retain(|record| record.id != event.entity || record.dimension != event.dimension);
            if dragons.len() >= TRACK_CAP { dragons.pop_front(); }
            dragons.push_back(Dragon {
                id: event.entity,
                dimension: event.dimension.to_owned(),
                left: None,
            });
        }
    }

    pub fn left(&self, event: &EntityLeaveLevelEvent<'_>) {
        if event.kind != "minecraft:ender_dragon" { return; }
        if let Ok(mut dragons) = self.dragons.lock() {
            for dragon in dragons.iter_mut() {
                if dragon.id == event.entity && dragon.dimension == event.dimension {
                    dragon.left = Some(self.tick.load(Ordering::Relaxed));
                }
            }
        }
    }

    pub fn died(&self, host: &mut dyn ServerHost, event: &mut VanillaGameEventEvent<'_>) {
        let Some(cause) = event.cause else { return; };
        if event.event != "minecraft:entity_die" { return; }
        let dragon = self.dragons.lock().ok().and_then(|mut dragons| {
            let index = dragons.iter().position(|d| {
                d.dimension == event.dimension && death_event(event.event, Some(cause), d.id)
            })?;
            dragons.remove(index)
        });
        if dragon.is_none() || !on(host, "dragon_drops_elytra") { return; }
        let Some(level) = host.level(event.dimension) else { return; };
        // The special dragon event has no killing-damage source. The original
        // prefers that source and otherwise takes the first nearby player.
        let player = host.players().server_players().into_iter().find(|player| {
            player.level(host).and_then(|l| l.dimension().id(host))
                .as_deref() == Some(event.dimension)
                && player.entity().position(host).is_some_and(|pos| inside(pos, event.pos))
        });
        let at = player.as_ref().and_then(|player| player.entity().position(host))
            .unwrap_or(event.pos);
        let at = [at[0].floor(), at[1].floor() + 1., at[2].floor()];
        let Some(id) = Identifier::parse("minecraft:elytra").ok() else { return; };
        let Some(item) = host.item_key(&id) else { return; };
        let Some(stack) = ItemStack::new(item, 1) else { return; };
        if !matches!(level.spawn_item(host, at[0], at[1], at[2], stack, 0), Ok(Some(_))) {
            return;
        }
        let broadcast = translate("collective.dragondropselytra.message.seemslikeslain",
            "The Ender Dragon has been slain.", "dark_green");
        for viewer in host.players().server_players() {
            if viewer.level(host).and_then(|l| l.dimension().id(host))
                .as_deref() == Some(event.dimension) {
                let _ = viewer.tell(host, broadcast.clone());
            }
        }
        if let Some(player) = player {
            let message = translate("collective.dragondropselytra.message.elytradroppedposition",
                "An Elytra has been dropped near you.", "yellow");
            let _ = player.tell(host, message);
        }
    }

    pub fn tick(&self, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) {
            return;
        }
        let now = self.tick.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        if let Ok(mut dragons) = self.dragons.lock() {
            dragons.retain(|dragon| dragon.left.is_none_or(|when| now.saturating_sub(when) <= REMOVE_GRACE));
        }
    }

    pub fn reset(&self) {
        self.tick.store(0, Ordering::Relaxed);
        if let Ok(mut dragons) = self.dragons.lock() { dragons.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dragon_game_event_is_exact_and_single_source() {
        assert!(death_event("minecraft:entity_die", Some(2), 2));
        assert!(!death_event("minecraft:entity_die", Some(1), 2));
        assert!(!death_event("minecraft:entity_die", None, 2));
        assert!(!death_event("minecraft:step", Some(2), 2));
        assert!(inside([30., 1., -30.], [0., 1., 0.]));
        assert!(!inside([31., 1., 0.], [0., 1., 0.]));
    }

    #[test]
    fn preserves_translation_ids_and_colours() {
        let text = translate("collective.dragondropselytra.message.seemslikeslain",
            "The Ender Dragon has been slain.", "dark_green");
        assert_eq!(text.style.color.as_deref(), Some("dark_green"));
        assert!(matches!(text.content, TextContent::Translatable { .. }));
    }
}
