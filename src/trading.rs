//! Completed merchant trades can restore their offer without touching game internals.
use mod_api::{ServerHost, TradeWithVillagerEvent};
use crate::settings::{flag, on};

pub fn on_trade(host: &mut dyn ServerHost, event: &mut TradeWithVillagerEvent<'_>) {
    if !on(host, "infinite_trading") { return; }
    let allowed = match event.kind.as_ref() {
        "minecraft:villager" => flag(host, "villagers", true),
        "minecraft:wandering_trader" => flag(host, "wandering_trader", true),
        _ => false,
    };
    if !allowed { return; }
    event.offer.uses = 0;
    if flag(host, "remove_demand", true) {
        event.offer.demand = 0;
    }
}
