//! Completed merchant trades can restore their offer without touching game internals.
use mod_api::{ServerHost, TradeWithVillagerEvent};
use crate::settings::{amount, flag, on};

pub fn on_trade(host: &mut dyn ServerHost, event: &mut TradeWithVillagerEvent<'_>) {
    if !on(host, "infinite_trading") { return; }
    let allowed = match event.kind.as_ref() {
        "minecraft:villager" if flag(host, "villagers", true) => {
            let required = amount(host, "minimumVillagerLevel", 1., 1., 5.) as i32;
            if required <= 1 {
                true
            } else {
                // The original checks VillagerData.level, not the career or
                // offer's XP. Unavailable merchants are never assumed high-level.
                host.entity(event.merchant)
                    .and_then(|entity| host.merchant(entity))
                    .and_then(|merchant| merchant.level(host))
                    .is_some_and(|level| level >= required)
            }
        }
        "minecraft:wandering_trader" => flag(host, "wandering_trader", true),
        _ => false,
    };
    if !allowed { return; }
    event.offer.uses = 0;
    if flag(host, "remove_demand", true) {
        event.offer.demand = 0;
    }
}
