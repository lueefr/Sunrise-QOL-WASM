//! Anvil cost changes through its public event; no direct Java/menu memory access.
use mod_api::{AnvilUpdateEvent, Flow, Host, HostSide};
use crate::settings::{amount, flag, on};

pub fn on_update(side: HostSide<'_>, event: &mut AnvilUpdateEvent<'_>) -> Flow {
    let host: &dyn Host = match &side {
        HostSide::Server(server) => &**server,
        HostSide::Client(client) => &**client,
    };
    // Combine matching enchanted books/items within the representable 255-level cap.
    crate::unlimited::anvil(host, event);
    if on(host, "tax_free_levels") && event.update.output.is_some() {
        // Only these two anvil behaviors are accessible from AnvilUpdateEvent.
        // The original experience-curve flattening is NOT implemented here.
        if event.update.right.is_none() && event.update.name.is_some()
            && flag(host, "taxfree_cheap_rename", true) {
            event.update.xp_cost = 1;
        }
        if flag(host, "taxfree_remove_anvil_limit", true) {
            event.update.xp_cost = event.update.xp_cost.clamp(1, 39);
        }
    }
    if !on(host, "affordable_anvil") { return Flow::Continue; }
    let cap = amount(host, "affordable_cost_cap", 39., 1., 10000.) as i32;
    if event.update.output.is_some() {
        if flag(host, "affordable_remove_too_expensive", true) {
            event.update.xp_cost = event.update.xp_cost.clamp(1, cap);
        }
        if flag(host, "affordable_cheap_rename", false)
            && event.update.right.is_none() && event.update.name.is_some() {
            event.update.xp_cost = 1;
        }
    }
    // The API does not recreate a result vanilla already discarded.
    Flow::Continue
}
