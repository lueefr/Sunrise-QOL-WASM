//! Vanilla Zoom uses Sunrise's camera zoom hook and an ordinary Controls binding.
//! Unlike the original Java mod this does not temporarily replace the offhand stack.
use crate::settings::{amount, flag, on};
use mod_api::{ClientHost, KeyConflictContext, KeyRegistration, Registrar, Zoom, ZoomEvent};

const KEY: &str = "qol_zoom";

pub fn register(r: &mut dyn Registrar) {
    r.register_key_mapping(
        KeyRegistration::new(KEY, "key.keyboard.left.alt")
            .label("qol.key.zoom")
            .context(KeyConflictContext::InGame),
    );
}

pub fn current(host: &dyn ClientHost, _: &mut ZoomEvent) -> Option<Zoom> {
    if !host.playing() || !on(host, "vanilla_zoom") || !host.is_down(crate::settings::ID, KEY) {
        return None;
    }
    Some(Zoom { factor: amount(host, "zoom_factor", 10., 2., 30.) as f32, smooth: flag(host, "zoom_smooth", true), turn: 0.15 })
}
