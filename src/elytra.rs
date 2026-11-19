//! Ground Elytra Fireworks: lift off, request vanilla fall flying, then use the held rocket.
//! The rocket is consumed only by Minecraft's ordinary use action once gliding begins.
use crate::settings::{amount, on};
use mod_api::{
    ClientHost, ClientTickPreEvent, Flow, HostSide, Identifier, RightClickItemEvent,
    ServerboundPlayPacket,
};
use std::sync::Mutex;

const TIMEOUT_MS: u64 = 2400;
const RETRY_MS: u64 = 180;
const MAX_TRIES: u8 = 6;

#[derive(Clone, Copy)]
struct Launch {
    begun: u64,
    last: u64,
    attempts: u8,
    player: u32,
}

#[derive(Default)]
pub struct Elytra {
    pending: Mutex<Option<Launch>>,
}

fn varint(mut value: u32, out: &mut Vec<u8>) {
    loop {
        if value & !0x7F == 0 {
            out.push(value as u8);
            break;
        }
        out.push((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
}

fn glide_packet(entity: u32) -> Option<ServerboundPlayPacket> {
    let mut body = Vec::with_capacity(7);
    varint(entity, &mut body);
    varint(6, &mut body); // START_FALL_FLYING action, Minecraft's player_command ordinal.
    varint(0, &mut body);
    Some(ServerboundPlayPacket::new(
        Identifier::parse("minecraft:player_command").ok()?,
        body,
    ))
}

impl Elytra {
    pub fn clicked(&self, side: HostSide<'_>, event: &mut RightClickItemEvent<'_>) -> Flow {
        let HostSide::Client(host) = side else { return Flow::Continue; };
        if !host.playing() || !on(host, "ground_elytra_fireworks")
            || event.click.stack.item().id(host) != "minecraft:firework_rocket" {
            return Flow::Continue;
        }
        let Some(player) = host.local_player() else { return Flow::Continue; };
        let entity = player.entity();
        if entity.on_ground(host) != Some(true)
            || entity.is_in_water(host) != Some(false)
            || host.living(entity).and_then(|living| living.is_gliding(host)) != Some(false) {
            return Flow::Continue;
        }
        let has_elytra = player.inventory_slots(host).is_some_and(|slots| {
            slots.iter().any(|(slot, stack)| {
                *slot == 38 && stack.item().id(host) == "minecraft:elytra"
            })
        });
        if !has_elytra { return Flow::Continue; }
        let Ok(mut pending) = self.pending.lock() else { return Flow::Continue; };
        if pending.is_some() { return Flow::Cancel; }
        let now = host.millis();
        let lift = amount(host, "elytra_launch_strength", 35., 20., 80.) as f32 / 100.;
        *pending = Some(Launch { begun: now, last: now, attempts: 0, player: entity.id() });
        host.push_player([0., lift, 0.]);
        Flow::Cancel
    }

    pub fn tick(&self, host: &mut dyn ClientHost, _: &mut ClientTickPreEvent) {
        let Ok(mut guard) = self.pending.lock() else { return; };
        let Some(mut pending) = *guard else { return; };
        if !on(host, "ground_elytra_fireworks") || !host.playing()
            || host.millis().saturating_sub(pending.begun) > TIMEOUT_MS {
            *guard = None;
            return;
        }
        let Some(player) = host.local_player() else { *guard = None; return; };
        let entity = player.entity();
        if entity.id() != pending.player { *guard = None; return; }
        if host.living(entity).and_then(|living| living.is_gliding(host)) == Some(true) {
            *guard = None;
            // Let the normal rocket use handle the boost and inventory consumption.
            let _ = host.click_mapping("key.use");
            return;
        }
        if entity.is_in_water(host) == Some(true) {
            *guard = None;
            return;
        }
        let now = host.millis();
        if entity.on_ground(host) == Some(false) && now.saturating_sub(pending.last) >= RETRY_MS
            && pending.attempts < MAX_TRIES {
            if let Some(packet) = glide_packet(entity.id()) {
                if host.send_packet(packet).is_ok() {
                    pending.attempts += 1;
                    pending.last = now;
                    *guard = Some(pending);
                } else {
                    *guard = None;
                }
            } else {
                *guard = None;
            }
        }
    }

    pub fn reset(&self) {
        if let Ok(mut pending) = self.pending.lock() { *pending = None; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_glide_packet_contents() {
        let packet = glide_packet(300).expect("packet");
        assert_eq!(packet.body, vec![0xAC, 0x02, 0x06, 0x00]);
        assert_eq!(MAX_TRIES, 6);
    }
}
