//! Refill depleted hand stacks after a use, preserving the replacement's data.
use crate::settings::on;
use mod_api::{Hand, ItemStack, ServerHost, ServerTickEvent, ServerTickPhase};
use std::sync::Mutex;

struct Pending { player: u128, slot: u32, used: ItemStack, ttl: u8 }
#[derive(Default)]
pub struct Refill { pending: Mutex<Vec<Pending>> }
impl Refill {
    pub fn arm(&self, host: &dyn ServerHost, hand: Hand, stack: Option<&ItemStack>) {
        if !on(host, "stack_refill") { return; }
        let Some(stack) = stack else { return; };
        let Some(player) = host.players().acting_player() else { return; };
        if player.is_creative(host) != Some(false) { return; }
        let slot = match hand { Hand::Off => 40, Hand::Main => match player.selected_slot(host) {
            Some(s) => s as u32, None => return,
        }};
        if let Ok(mut pending) = self.pending.lock() {
            pending.retain(|p| p.player != player.uuid() || p.slot != slot);
            pending.push(Pending { player: player.uuid(), slot, used: stack.clone(), ttl: 60 });
        }
    }
    pub fn tick(&self, host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) { return; }
        let Ok(mut queue) = self.pending.lock() else { return; };
        if !on(host, "stack_refill") { queue.clear(); return; }
        queue.retain_mut(|pending| {
            pending.ttl = pending.ttl.saturating_sub(1);
            let Some(player) = host.player(pending.player) else { return false; };
            if pending.ttl == 0 || (pending.slot != 40 && player.selected_slot(host) != Some(pending.slot as usize)) {
                return false;
            }
            if host.living(player.entity()).is_none_or(|l| l.alive(host) != Some(true)) { return false; }
            let Some(slots) = player.inventory_slots(host) else { return false; };
            if let Some((_, held)) = slots.iter().find(|(slot, _)| *slot == pending.slot) {
                return held.same_item(&pending.used);
            }
            // Preserve components: do not silently substitute another potion, map or named stack.
            let source = slots.into_iter().filter(|(slot, stack)| *slot >= 9 && *slot <= 35
                && stack.is_stackable_with(&pending.used)).max_by_key(|(slot, _)| *slot);
            let Some((slot, stack)) = source else { return false; };
            if player.set_inventory_slot(host, slot, None).is_err() { return false; }
            if player.set_inventory_slot(host, pending.slot, Some(stack.clone())).is_err() {
                let _ = player.set_inventory_slot(host, slot, Some(stack));
            }
            false
        });
    }
    pub fn reset(&self) { if let Ok(mut pending) = self.pending.lock() { pending.clear(); } }
}
