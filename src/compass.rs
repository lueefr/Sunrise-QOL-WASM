//! GUI Compass displays the local player's facing and block position in the HUD.
//! Settings are cached briefly to avoid repeated config-file reads every rendered frame.
use crate::settings::{amount, flag, on, saved_value};
use mod_api::{ClientHost, GuiRenderPostEvent};
use std::sync::Mutex;

const DIRECTIONS: [&str; 8] = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
const REFRESH_MS: u64 = 500;

#[derive(Clone)]
struct Options {
    enabled: bool,
    compass_required: bool,
    format: String,
    position: String,
    offset: f32,
    shadow: bool,
    color: u32,
}

impl Options {
    fn load(host: &dyn ClientHost) -> Self {
        Self {
            enabled: on(host, "gui_compass"),
            compass_required: flag(host, "mustHaveCompassInInventory", true),
            format: saved_value(host, "guiCompassFormat").unwrap_or_else(|| "FXYZ".to_owned()),
            position: saved_value(host, "compassPosition").unwrap_or_else(|| "Left".to_owned()),
            offset: amount(host, "compassHeightOffset", 5., 0., 3000.) as f32,
            shadow: flag(host, "drawTextShadow", true),
            color: color(host),
        }
    }
}

#[derive(Default)]
pub struct Compass {
    settings: Mutex<Option<(u64, Options)>>,
}

fn facing(yaw: f32) -> &'static str {
    if !yaw.is_finite() { return "?"; }
    let octant = ((yaw.rem_euclid(360.0) / 45.0).round() as usize) % 8;
    DIRECTIONS[octant]
}

fn coords(yaw: f32, position: [f64; 3], format: &str) -> String {
    let [x, y, z] = position.map(|component| component.floor() as i32);
    match format {
        "F" => facing(yaw).to_owned(),
        "XYZ" => format!("{x}, {y}, {z}"),
        _ => format!("{}: {x}, {y}, {z}", facing(yaw)),
    }
}

fn color(host: &dyn ClientHost) -> u32 {
    let rgb = saved_value(host, "compassRGB")
        .unwrap_or_else(|| "255,255,255".to_owned());
    let mut channels = [255_u8; 3];
    for (index, text) in rgb.split(',').take(3).enumerate() {
        if let Ok(channel) = text.trim().parse::<u8>() { channels[index] = channel; }
    }
    for (index, key) in ["compass_red", "compass_green", "compass_blue"].iter().enumerate() {
        if let Some(value) = saved_value(host, key).and_then(|v| v.parse::<u8>().ok()) {
            channels[index] = value;
        }
    }
    0xFF00_0000 | (u32::from(channels[0]) << 16)
        | (u32::from(channels[1]) << 8) | u32::from(channels[2])
}

impl Compass {
    pub fn render(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if event.gui_hidden || !host.playing() { return; }
        let now = host.millis();
        let options = {
            let Ok(mut lock) = self.settings.lock() else { return; };
            if lock.as_ref().is_none_or(|(read, _)| now.saturating_sub(*read) >= REFRESH_MS) {
                *lock = Some((now, Options::load(host)));
            }
            let Some((_, options)) = lock.as_ref() else { return; };
            options.clone()
        };
        if !options.enabled { return; }
        let Some(player) = host.local_player() else { return; };
        if options.compass_required
            && !player.inventory_slots(host).is_some_and(|items| {
                items.iter().any(|(_, stack)| stack.item().id(host) == "minecraft:compass")
            }) { return; }
        let entity = player.entity();
        let (Some(position), Some(rotation)) = (entity.position(host), entity.rotation(host)) else { return; };
        let content = coords(rotation[0], position, &options.format);
        let [width, height] = event.batch.size();
        if options.offset > height - 9.0 { return; }
        let text_width = event.batch.text_width(&content);
        let x = match options.position.as_str() {
            "Right" => (width - text_width - 5.).max(0.),
            "Center" => ((width - text_width) / 2.).max(0.),
            _ => 5.,
        };
        event.batch.text(&content, [x, options.offset], options.color, options.shadow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directions_match_original_yaw_order() {
        assert_eq!(facing(0.), "S");
        assert_eq!(facing(45.), "SW");
        assert_eq!(facing(90.), "W");
        assert_eq!(facing(180.), "N");
        assert_eq!(facing(-90.), "E");
        assert_eq!(facing(360.), "S");
    }
    #[test]
    fn negative_coordinates_use_block_flooring() {
        assert_eq!(coords(0., [-0.2, 64.9, -2.1], "XYZ"), "-1, 64, -3");
        assert_eq!(coords(90., [1., 2., 3.], "F"), "W");
    }
}
