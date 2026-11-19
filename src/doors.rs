//! Double Doors synchronizes manual clicks in the interaction callback.
//! Non-player changes use a bounded deferred queue because block_changed is read-only.
use crate::settings::{amount, flag, on};
use mod_api::{
    BlockChangedEvent, BlockUpdateFlags, Flow, Hand, HostSide, LevelHost,
    LevelRef, RightClickBlockEvent, ServerHost, ServerTickEvent, ServerTickPhase,
};
use std::collections::{BTreeSet, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_PENDING: usize = 256;
const MAX_CONNECTED: usize = 256;
const PER_TICK: usize = 4;

#[derive(Clone)]
struct Change {
    dimension: String,
    pos: [i32; 3],
    name: String,
    open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind { Door, Trapdoor, Gate }

fn kind(name: &str) -> Option<Kind> {
    let path = name.rsplit_once(':').map_or(name, |(_, path)| path);
    if path.ends_with("_trapdoor") { Some(Kind::Trapdoor) }
    else if path.ends_with("_fence_gate") { Some(Kind::Gate) }
    else if path.ends_with("_door") { Some(Kind::Door) }
    else { None }
}

fn prop<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let props = state.split_once('[')?.1.strip_suffix(']')?;
    props.split(',').find_map(|p| {
        let (name, value) = p.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn replace(state: &str, key: &str, value: &str) -> Option<String> {
    let (name, props) = state.split_once('[')?;
    let props = props.strip_suffix(']')?;
    let mut found = false;
    let parts: Vec<String> = props.split(',').map(|part| {
        if part.split_once('=').is_some_and(|(k, _)| k == key) {
            found = true;
            format!("{key}={value}")
        } else { part.to_owned() }
    }).collect();
    found.then(|| format!("{name}[{}]", parts.join(",")))
}

fn neighbor(pos: [i32; 3], x: i32, y: i32, z: i32) -> [i32; 3] {
    [pos[0] + x, pos[1] + y, pos[2] + z]
}

fn base(state: &str, mut pos: [i32; 3], class: Kind) -> [i32; 3] {
    if class == Kind::Door && prop(state, "half") == Some("upper") { pos[1] -= 1; }
    pos
}

fn within(origin: [i32; 3], pos: [i32; 3], radius: i32) -> bool {
    let squared: i64 = (0..3).map(|i| {
        let delta = i64::from(pos[i]) - i64::from(origin[i]);
        delta * delta
    }).sum();
    squared <= i64::from(radius) * i64::from(radius)
}

#[derive(Default)]
pub struct Doors {
    pending: Mutex<VecDeque<Change>>,
    bypass: Mutex<Vec<(String, [i32; 3])>>,
    writing: AtomicBool,
}

struct WriteGuard<'a>(&'a AtomicBool);
impl Drop for WriteGuard<'_> {
    fn drop(&mut self) { self.0.store(false, Ordering::Release); }
}

impl Doors {
    // Ignore the click's later block_changed notice: it is already handled here.
    // The same marker also lets sneaking preserve vanilla's single-door behavior.
    fn bypass_click(&self, dim: &str, pos: [i32; 3]) {
        if let Ok(mut bypass) = self.bypass.lock() {
            if bypass.len() >= 192 { bypass.clear(); }
            for dy in [-1, 0, 1] {
                if let Some(y) = pos[1].checked_add(dy) {
                    bypass.push((dim.to_owned(), [pos[0], y, pos[2]]));
                }
            }
        }
    }

    // right_click_block fires before vanilla handles the door. Update the
    // matching neighbors now, not one server tick after the first door opens.
    // The clicked door is left to vanilla so normal sounds and interaction
    // semantics are preserved. Other state changes still use the fallback queue.
    pub fn clicked(&self, side: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        let HostSide::Server(host) = side else { return Flow::Continue; };
        if !on(host, "double_doors") || event.click.hand != Hand::Main { return Flow::Continue; }
        let Some(player) = host.players().acting_player() else { return Flow::Continue; };
        let dim = event.click.dimension.key().id().to_owned();
        let pos = event.click.pos;
        if player.is_secondary_use_active(host) == Some(true) {
            self.bypass_click(&dim, pos);
            return Flow::Continue;
        }
        let Some(level) = host.level(&dim) else { return Flow::Continue; };
        if !level.in_place(host) { return Flow::Continue; }
        let Some(state) = level.block_state(host, pos) else { return Flow::Continue; };
        let Some(text) = host.block_state_string(state) else { return Flow::Continue; };
        let name = text.split_once('[').map_or(text.as_str(), |(name, _)| name);
        let Some(class) = kind(name) else { return Flow::Continue; };
        // Vanilla cannot open iron doors by hand. Avoid simulating a click
        // that the underlying block would reject.
        if class == Kind::Door && name == "minecraft:iron_door" {
            return Flow::Continue;
        }
        if class == Kind::Gate && !flag(host, "gates", false) { return Flow::Continue; }
        if class == Kind::Trapdoor && !flag(host, "trapdoors", false) {
            return Flow::Continue;
        }
        let open = match prop(&text, "open") {
            Some("true") => false,
            Some("false") => true,
            _ => return Flow::Continue,
        };
        let pos = base(&text, pos, class);
        let task = Change { dimension: dim.clone(), pos, name: name.to_owned(), open };
        if self.writing.swap(true, Ordering::AcqRel) { return Flow::Continue; }
        {
            let _guard = WriteGuard(&self.writing);
            self.process(host, task);
        }
        self.bypass_click(&dim, pos);
        Flow::Continue
    }

    // This hook has read-only access; defer all paired writes until the next tick.
    pub fn changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        if self.writing.load(Ordering::Acquire) { return; }
        if event.old.block() != event.new.block() { return; }
        let (Some(old), Some(new)) = (level.state_string(event.old), level.state_string(event.new)) else { return; };
        let (Some(before), Some(after)) = (prop(&old, "open"), prop(&new, "open")) else { return; };
        if before == after { return; }
        let name = new.split_once('[').map_or(new.as_str(), |(name, _)| name);
        let Some(class) = kind(name) else { return; };
        let dim = event.dimension.key().id();
        let pos = base(&new, [event.pos.x, event.pos.y, event.pos.z], class);
        if self.bypass.lock().is_ok_and(|list| list.iter().any(|(d, p)| d == dim && *p == pos)) {
            return;
        }
        if let Ok(mut pending) = self.pending.lock() {
            if pending.len() >= MAX_PENDING { return; }
            if let Some(task) = pending.iter_mut().find(|t| t.dimension == dim && t.pos == pos) {
                task.open = after == "true";
            } else {
                pending.push_back(Change { dimension: dim.to_owned(), pos,
                    name: name.to_owned(), open: after == "true" });
            }
        }
    }

    fn set_open(&self, host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3],
                name: &str, class: Kind, open: bool, facing: Option<&str>) {
        let Some(state) = level.block_state(host, pos) else { return; };
        let Some(text) = host.block_state_string(state) else { return; };
        if !text.starts_with(name) || text.as_bytes().get(name.len()) != Some(&b'[') { return; }
        let target = if open { "true" } else { "false" };
        let Some(mut modified) = replace(&text, "open", target) else { return; };
        if let (Kind::Gate, Some(facing)) = (class, facing) {
            if let Some(candidate) = replace(&modified, "facing", facing) { modified = candidate; }
        }
        if modified == text { return; }
        let Some(next) = host.block_state_id(&modified) else { return; };
        let _ = level.set_block(host, pos, next, BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS);
        if class == Kind::Door {
            let upper = neighbor(pos, 0, 1, 0);
            if let Some(state) = level.block_state(host, upper) {
                if let Some(text) = host.block_state_string(state) {
                    if text.starts_with(name) && prop(&text, "half") == Some("upper") {
                        if let Some(modified) = replace(&text, "open", target) {
                            if let Some(next) = host.block_state_id(&modified) {
                                let _ = level.set_block(host, upper, next, BlockUpdateFlags::CLIENTS);
                            }
                        }
                    }
                }
            }
        }
    }

    fn process(&self, host: &mut dyn ServerHost, task: Change) {
        if !on(host, "double_doors") { return; }
        let Some(class) = kind(&task.name) else { return; };
        if class == Kind::Gate && !flag(host, "gates", false) { return; }
        if class == Kind::Trapdoor && !flag(host, "trapdoors", false) { return; }
        let Some(level) = host.level(&task.dimension) else { return; };
        if !level.in_place(host) { return; }
        let Some(state) = level.block_state(host, task.pos) else { return; };
        let Some(text) = host.block_state_string(state) else { return; };
        if !text.starts_with(&task.name) || prop(&text, "open") != Some(if task.open { "true" } else { "false" }) {
            return;
        }
        let recursive = flag(host, "recursive_doors", true);
        let radius = amount(host, "door_radius", 10., 1., 64.) as i32;
        let source_face = prop(&text, "facing").map(str::to_owned);
        let mut seen = BTreeSet::from([task.pos]);
        let mut queue = VecDeque::from([task.pos]);
        while let Some(pos) = queue.pop_front() {
            if seen.len() >= MAX_CONNECTED { break; }
            let dy = if class == Kind::Door { 0 } else { 1 };
            for y in -dy..=dy {
                for x in -1..=1 {
                    for z in -1..=1 {
                        if x == 0 && y == 0 && z == 0 { continue; }
                        let next = neighbor(pos, x, y, z);
                        if seen.contains(&next) || !within(task.pos, next, radius) { continue; }
                        let Some(state) = level.block_state(host, next) else { continue; };
                        let Some(text) = host.block_state_string(state) else { continue; };
                        if !text.starts_with(&task.name) || text.as_bytes().get(task.name.len()) != Some(&b'[') {
                            continue;
                        }
                        if class == Kind::Door && prop(&text, "half") == Some("upper") { continue; }
                        if prop(&text, "open").is_none() { continue; }
                        if class == Kind::Gate && source_face.as_deref().is_some_and(|face| {
                            let other = prop(&text, "facing");
                            let opposite = match face { "north" => "south", "south" => "north",
                                "east" => "west", "west" => "east", _ => face };
                            other != Some(face) && other != Some(opposite)
                        }) { continue; }
                        seen.insert(next);
                        if recursive { queue.push_back(next); }
                        self.set_open(host, level, next, &task.name, class, task.open, source_face.as_deref());
                        if seen.len() >= MAX_CONNECTED { break; }
                    }
                }
            }
        }
    }

    pub fn tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { world_running: true, .. }) { return; }
        if let Ok(mut bypass) = self.bypass.lock() { bypass.clear(); }
        if !on(host, "double_doors") {
            if let Ok(mut pending) = self.pending.lock() { pending.clear(); }
            return;
        }
        for _ in 0..PER_TICK {
            let Some(task) = self.pending.lock().ok().and_then(|mut q| q.pop_front()) else { break; };
            if self.writing.swap(true, Ordering::AcqRel) { break; }
            let _guard = WriteGuard(&self.writing);
            self.process(host, task);
        }
    }

    pub fn reset(&self) {
        if let Ok(mut pending) = self.pending.lock() { pending.clear(); }
        if let Ok(mut bypass) = self.bypass.lock() { bypass.clear(); }
        self.writing.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_only_actual_door_families() {
        assert_eq!(kind("minecraft:oak_door"), Some(Kind::Door));
        assert_eq!(kind("create:andesite_trapdoor"), Some(Kind::Trapdoor));
        assert_eq!(kind("minecraft:oak_fence_gate"), Some(Kind::Gate));
        assert_eq!(kind("minecraft:stone"), None);
    }
    #[test]
    fn rewrites_only_the_open_property() {
        let original = "minecraft:oak_door[facing=east,half=lower,hinge=left,open=false,powered=false]";
        let next = replace(original, "open", "true").unwrap();
        assert_eq!(prop(&next, "open"), Some("true"));
        assert_eq!(prop(&next, "hinge"), Some("left"));
        assert_eq!(replace(original, "unknown", "true"), None);
        assert_eq!(base("minecraft:oak_door[half=upper,open=true]", [1, 81, 2], Kind::Door), [1,80,2]);
    }
    #[test]
    fn bounded_radius_does_not_overflow() {
        assert!(within([0, 64, 0], [6, 72, 0], 10));
        assert!(!within([0, 64, 0], [8, 72, 0], 10));
        assert!(!within([0, 64, 0], [64, 64, 64], 64));
    }
}
