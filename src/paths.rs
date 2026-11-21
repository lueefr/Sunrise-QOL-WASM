//! Select a start with sneak + shovel, click an end to create a surface route.
use crate::settings::on;
use mod_api::{BlockUpdateFlags, EquipmentSlot, Flow, Hand, RightClickBlockEvent, ServerHost};
use std::sync::Mutex;

struct Route { player: u128, dimension: String, start: [i32;3], end: [i32;3], made: Vec<[i32;3]> }
#[derive(Default)]
pub struct Paths { routes: Mutex<Vec<Route>> }
impl Paths {
    pub fn click(&self, host: &mut dyn ServerHost, event: &RightClickBlockEvent<'_>) -> Flow {
        if !on(host, "quick_paths") { return Flow::Continue; }
        let Some(stack) = event.click.stack else { return Flow::Continue; };
        if !stack.item().id(host).ends_with("_shovel") { return Flow::Continue; }
        let Some(player) = host.players().acting_player() else { return Flow::Continue; };
        if player.may_build(host) != Some(true) { return Flow::Continue; }
        let Some(level) = player.level(host) else { return Flow::Continue; };
        let Some(dimension) = player.dimension(host).map(|d| d.to_string()) else { return Flow::Continue; };
        if !level.in_place(host) { return Flow::Continue; }
        let pos = event.click.pos;
        let Some(block) = level.block(host, pos) else { return Flow::Continue; };
        let name = block.id(host);
        let Ok(mut routes) = self.routes.lock() else { return Flow::Continue; };
        let own = routes.iter().position(|r| r.player == player.uuid());
        let flags = BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS;
        if name == "minecraft:dirt_path" {
            if let Some(index) = own.filter(|i| routes[*i].dimension == dimension && routes[*i].end == pos) {
                let route = routes.remove(index);
                if let Some(grass) = host.block_state_id("minecraft:grass_block") {
                    for at in route.made {
                        if level.block(host, at).is_some_and(|b| b.id(host) == "minecraft:dirt_path") {
                            let _ = level.set_block(host, at, grass, flags);
                        }
                    }
                }
                return Flow::Cancel;
            }
        }
        if name != "minecraft:grass_block" { return Flow::Continue; }
        if player.is_secondary_use_active(host) == Some(true) {
            if let Some(index) = own { routes.remove(index); }
            routes.push(Route { player: player.uuid(), dimension, start: pos, end: pos, made: Vec::new() });
            return Flow::Cancel;
        }
        let Some(index) = own.filter(|i| routes[*i].dimension == dimension && routes[*i].made.is_empty()) else { return Flow::Continue; };
        let start = routes[index].start;
        let distance = (i64::from(pos[0])-i64::from(start[0])).abs() + (i64::from(pos[2])-i64::from(start[2])).abs();
        if distance > 128 { return Flow::Cancel; }
        let Some(path) = host.block_state_id("minecraft:dirt_path") else { return Flow::Continue; };
        let mut columns = vec![[start[0], start[2]]];
        let mut at = columns[0];
        // A bounded staircase joins the selected columns, including diagonals.
        while at != [pos[0], pos[2]] {
            if at[0] != pos[0] { at[0] += (pos[0]-at[0]).signum(); columns.push(at); }
            if at[1] != pos[2] { at[1] += (pos[2]-at[1]).signum(); columns.push(at); }
        }
        let mut made = Vec::new();
        for [x,z] in columns {
            let target = (0i32..=10).flat_map(|delta| [delta, -delta]).find_map(|dy| {
                let y = start[1].checked_add(dy)?;
                let above = y.checked_add(1)?;
                let at = [x,y,z];
                (level.block(host, at).is_some_and(|b| b.id(host) == "minecraft:grass_block")
                    && level.block(host, [x,above,z]).is_some_and(|b| b.id(host) == "minecraft:air")).then_some(at)
            });
            if let Some(at) = target {
                if level.set_block(host, at, path, flags) == Ok(true) { made.push(at); }
            }
        }
        // Extra path-building wear must respect Unbreakable Tools as well.
        if player.is_creative(host) != Some(true) && !on(host, "what_durability") {
            if let Some(living) = host.living(player.entity()) {
                let slot = if event.click.hand == Hand::Main { EquipmentSlot::MainHand } else { EquipmentSlot::OffHand };
                let _ = living.hurt_and_break(host, slot, made.len() as i32);
            }
        }
        routes[index].end = pos;
        routes[index].made = made;
        Flow::Cancel
    }
    pub fn reset(&self) { if let Ok(mut routes) = self.routes.lock() { routes.clear(); } }
}
