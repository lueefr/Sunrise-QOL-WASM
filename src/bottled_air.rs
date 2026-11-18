//! Bottled Air: a vanilla glass bottle restores configurable breath underwater.
//! The vanilla item use continues, so the bottle still fills water normally.
use crate::settings::{amount, on};
use mod_api::{Flow, HostSide, RightClickItemEvent};

pub fn use_bottle(side: HostSide<'_>, event: &mut RightClickItemEvent<'_>) -> Flow {
    let HostSide::Server(host) = side else { return Flow::Continue; };
    if !on(host, "bottled_air") { return Flow::Continue; }
    if event.click.stack.item().id(host) != "minecraft:glass_bottle" {
        return Flow::Continue;
    }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    let entity = player.entity();
    if entity.is_in_water(host) != Some(true) { return Flow::Continue; }
    let (Some(current), Some(max)) = (entity.air_supply(host), entity.max_air_supply(host)) else {
        return Flow::Continue;
    };
    if current >= max { return Flow::Continue; }
    let restored = amount(host, "bottled_air_amount", 150., 0., 1000.) as i32;
    if restored <= 0 { return Flow::Continue; }
    let next = current.saturating_add(restored).min(max);
    let _ = entity.set_air_supply(host, next);
    Flow::Continue
}

#[cfg(test)]
mod tests {
    #[test]
    fn capped_air_restoration() {
        let current: i32 = 210;
        let max: i32 = 300;
        let restored: i32 = 150;
        assert_eq!(current.saturating_add(restored).min(max), 300);
        assert_eq!((-20_i32).saturating_add(restored).min(max), 130);
    }
}
