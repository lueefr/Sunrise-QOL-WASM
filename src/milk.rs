//! Nutritious Milk: the original adjusts hunger and saturation after drinking
//! any registered item whose id contains "milk_bucket".
use crate::settings::{amount, on};
use mod_api::{HostSide, LivingEntityUseItemFinishEvent};

fn is_milk(item: &str) -> bool {
    item.contains("milk_bucket")
}

pub fn finished(side: HostSide<'_>, event: &mut LivingEntityUseItemFinishEvent<'_>) {
    let HostSide::Server(host) = side else { return; };
    if !on(host, "nutritious_milk") || !is_milk(&event.stack.item().id(host)) {
        return;
    }
    let Some(player) = host.players().acting_player() else { return; };
    let Some(food) = player.food(host) else { return; };
    let hunger = amount(host, "hungerLevelIncrease", 10., 0., 20.) as i32;
    let saturation = amount(host, "saturationLevelIncrease", 10., 0., 20.) as f32;
    // Vanilla's FoodData setters take the result directly, without clamping
    // the amount in the original mod to the remaining hunger/saturation gap.
    let _ = player.set_food_level(host, food.level.saturating_add(hunger));
    let _ = player.set_saturation(host, food.saturation + saturation);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_milk_id_filter() {
        assert!(is_milk("minecraft:milk_bucket"));
        assert!(is_milk("example:goat_milk_bucket"));
        assert!(!is_milk("minecraft:bucket"));
        assert!(!is_milk("minecraft:honey_bottle"));
    }
}
