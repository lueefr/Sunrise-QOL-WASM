//! Restore bookshelf offer levels when water blocks the vanilla power path.
use crate::settings::on;
use mod_api::{EnchantmentLevelSetEvent, ServerHost};

pub fn offer(host: &mut dyn ServerHost, event: &mut EnchantmentLevelSetEvent<'_>) {
    if !on(host, "underwater_enchanting") { return; }
    let Some(player) = host.player(event.offer.player) else { return; };
    let Some(level) = player.level(host) else { return; };
    if !level.in_place(host) { return; }
    let mut shelves = 0;
    let mut water = false;
    for x in -2i32..=2 { for z in -2i32..=2 {
        if x.abs() != 2 && z.abs() != 2 { continue; }
        for y in 0..=1 {
            let [tx,ty,tz] = event.pos;
            let (Some(sx),Some(sy),Some(sz),Some(mx),Some(mz)) = (
                tx.checked_add(x),ty.checked_add(y),tz.checked_add(z),tx.checked_add(x/2),tz.checked_add(z/2)
            ) else { continue; };
            if level.block(host, [sx,sy,sz]).is_none_or(|b| b.id(host) != "minecraft:bookshelf") { continue; }
            let Some(middle) = level.block(host, [mx,sy,mz]) else { continue; };
            match middle.id(host).as_str() {
                "minecraft:water" => { shelves += 1; water = true; },
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => shelves += 1,
                _ => {},
            }
        }
    }}
    let power = shelves.min(15);
    if !water || power <= event.offer.power { return; }
    // The original changes power detection. The public hook instead supplies
    // the equivalent cost range before the native enchantment list is rolled.
    let Some(base) = level.with_random(host, |rng| rng.next_int_bound(8) + 1 + power / 2 + rng.next_int_bound(power + 1)) else { return; };
    event.level = match event.offer.row {
        0 => (base / 3).max(1),
        1 => base * 2 / 3 + 1,
        2 => base.max(power * 2),
        _ => return,
    };
}
