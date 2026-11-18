//! Slime-join veto on a world the server owner explicitly marked superflat.
//! Sunrise's WASM API exposes no reliable world-preset read; never guess from
//! terrain blocks or use a folder path unavailable inside the sandbox.
use crate::settings::{flag, on};
use mod_api::{EntityJoinLevelEvent, Flow, ServerHost};

pub fn on_join(host: &dyn ServerHost, event: &EntityJoinLevelEvent<'_>) -> Flow {
    if event.kind == "minecraft:slime"
        && on(host, "superflat_world_no_slimes")
        && flag(host, "confirm_superflat", false)
    {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}
