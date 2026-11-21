//! Dragon Drops Elytra uses the dragon-specific death game event after the
//! vanilla death animation, not the ordinary mob loot pipeline.
use crate::settings::on;
use mod_api::{
    EntityJoinLevelEvent, EntityLeaveLevelEvent, Identifier, ItemStack, LivingDamagePostEvent,
    ServerHost, ServerTickEvent, ServerTickPhase, Text, VanillaGameEventEvent,
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
    cause: Option<i32>,
}

#[derive(Default)]
pub struct DragonDrops {
    tick: AtomicU64,
    dragons: Mutex<VecDeque<Dragon>>,
}

fn death_event(kind: &str, entity: Option<u32>, tracked: u32) -> bool {
    kind == "minecraft:entity_die" && entity == Some(tracked)
}

fn bounds(pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let origin = pos.map(f64::floor);
    ([origin[0] - 30., origin[1] - 30., origin[2] - 30.],
     [origin[0] + 30., origin[1] + 30., origin[2] + 30.])
}

fn recipient(source: Option<i32>, players: &[u32], nearby: &[u32]) -> Option<u32> {
    match source.filter(|id| *id != 0) {
        // The original does not use nearby fallback when a non-player dealt
        // the final blow. A distant player who dealt it still takes priority.
        Some(id) => u32::try_from(id).ok().filter(|id| players.contains(id)),
        None => nearby.iter().copied().find(|id| players.contains(id)),
    }
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
                cause: None,
            });
        }
    }

    pub fn damage(&self, event: &LivingDamagePostEvent<'_>) {
        if event.kind != "minecraft:ender_dragon" || event.container.health_damage() <= 0. {
            return;
        }
        let Ok(id) = u32::try_from(event.entity) else { return; };
        if let Ok(mut dragons) = self.dragons.lock() {
            if let Some(dragon) = dragons.iter_mut().find(|dragon| dragon.id == id) {
                // A later damaging hit replaces the preceding source, including
                // an environmental (zero-causer) killing blow.
                dragon.cause = Some(event.source.causing);
            }
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
        let Some(dragon) = dragon else { return; };
        if !on(host, "dragon_drops_elytra") { return; }
        let Some(level) = host.level(event.dimension) else { return; };
        let (min, max) = bounds(event.pos);
        let nearby = if dragon.cause.unwrap_or(0) == 0 {
            // Preserve the world's entity-query order rather than the global
            // online-player order used by the former implementation.
            level.entities_in(host, min, max).unwrap_or_default()
                .into_iter().map(|entity| entity.id()).collect()
        } else {
            Vec::new()
        };
        let players = host.players().server_players();
        let player_ids: Vec<u32> = players.iter().map(|player| player.entity().id()).collect();
        let target = recipient(dragon.cause, &player_ids, &nearby);
        let player = players.iter().find(|player| Some(player.entity().id()) == target);
        let at = player.and_then(|player| player.entity().position(host))
            .unwrap_or(event.pos);
        let at = [at[0].floor(), at[1].floor() + 1., at[2].floor()];

        // The Java source broadcasts before attempting the item spawn.
        let broadcast = translate("collective.dragondropselytra.message.seemslikeslain",
            "The Ender Dragon has been slain.", "dark_green");
        for viewer in &players {
            if viewer.level(host).and_then(|l| l.dimension().id(host))
                .as_deref() == Some(event.dimension) {
                let _ = viewer.tell(host, broadcast.clone());
            }
        }
        if let Ok(id) = Identifier::parse("minecraft:elytra") {
            if let Some(item) = host.item_key(&id) {
                if let Some(stack) = ItemStack::new(item, 1) {
                    let _ = level.spawn_item(host, at[0], at[1], at[2], stack, 0);
                }
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
        assert_eq!(bounds([4.9, 70.8, -2.1]),
            ([-26., 40., -33.], [34., 100., 27.]));
    }

    #[test]
    fn original_player_priority_and_nonplayer_semantics() {
        let players = [2, 4, 6];
        let nearby = [6, 4];
        assert_eq!(recipient(Some(2), &players, &nearby), Some(2));
        assert_eq!(recipient(None, &players, &nearby), Some(6));
        assert_eq!(recipient(Some(0), &players, &nearby), Some(6));
        assert_eq!(recipient(Some(9), &players, &nearby), None);
        assert_eq!(recipient(None, &players, &[]), None);
    }

    #[test]
    fn preserves_translation_ids_and_colours() {
        let text = translate("collective.dragondropselytra.message.seemslikeslain",
            "The Ender Dragon has been slain.", "dark_green");
        assert_eq!(text.style.color.as_deref(), Some("dark_green"));
        assert!(matches!(text.content, TextContent::Translatable { .. }));
    }
}
