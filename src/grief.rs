//! Villager griefing is tracked through entity lifecycle hooks, not native entity reads.
//! The global griefing hook only supplies `Host`; `entity_kind` requires `ServerHost`.
use crate::settings::on;
use mod_api::{EntityJoinLevelEvent, EntityLeaveLevelEvent, EntityMobGriefingEvent, Flow, Host};
use std::sync::Mutex;

const MAX_TRACKED: usize = 65536;

#[derive(Default)]
pub struct Villagers {
    // Sorted IDs permit binary search without a hash table in the WASM guest.
    ids: Mutex<Vec<u32>>,
}

impl Villagers {
    pub fn on_join(&self, event: &EntityJoinLevelEvent<'_>) -> Flow {
        if event.kind != "minecraft:villager" {
            return Flow::Continue;
        }
        let mut ids = self.ids.lock().unwrap_or_else(|error| error.into_inner());
        match ids.binary_search(&event.entity) {
            Ok(_) => {}
            Err(index) if ids.len() < MAX_TRACKED => ids.insert(index, event.entity),
            Err(_) => {}
        }
        Flow::Continue
    }

    pub fn on_leave(&self, event: &EntityLeaveLevelEvent<'_>) {
        if event.kind != "minecraft:villager" {
            return;
        }
        let mut ids = self.ids.lock().unwrap_or_else(|error| error.into_inner());
        if let Ok(index) = ids.binary_search(&event.entity) {
            ids.remove(index);
        }
    }

    pub fn on_grief(&self, host: &dyn Host, event: &mut EntityMobGriefingEvent) {
        if event.can_grief || !on(host, "friendly_griefing") {
            return;
        }
        let ids = self.ids.lock().unwrap_or_else(|error| error.into_inner());
        if ids.binary_search(&event.entity.id()).is_ok() {
            event.can_grief = true;
        }
    }

    pub fn reset(&self) {
        self.ids.lock().unwrap_or_else(|error| error.into_inner()).clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_ids_support_insert_and_remove() {
        let mut ids = Vec::new();
        for id in [7_u32, 2, 4, 2] {
            if let Err(index) = ids.binary_search(&id) {
                ids.insert(index, id);
            }
        }
        assert_eq!(ids, vec![2, 4, 7]);
        ids.remove(ids.binary_search(&4).unwrap());
        assert_eq!(ids, vec![2, 7]);
    }
}
