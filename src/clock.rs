//! GUI Clock displays the level's synchronized Overworld clock and elapsed days.
//! The sandbox exposes world time but not the computer's local wall-clock time.
use crate::settings::{amount, flag, on, saved_value};
use mod_api::{ClientHost, GuiRenderPostEvent};
use std::sync::Mutex;

const REFRESH_MS: u64 = 500;

#[derive(Clone)]
struct Options {
    enabled: bool,
    required: bool,
    hours24: bool,
    show_days: bool,
    minute_step: i64,
    position: String,
    top: f32,
    horizontal: f32,
    shadow: bool,
    color: u32,
}

impl Options {
    fn load(host: &dyn ClientHost) -> Self {
        let channel = |key: &str| amount(host, key, 255., 0., 255.) as u32;
        Self {
            enabled: on(host, "gui_clock"),
            required: flag(host, "mustHaveClockInInventoryForGameTime", true),
            hours24: flag(host, "clock24Hour", true),
            show_days: flag(host, "showDaysPlayedWorld", true),
            minute_step: amount(host, "minuteStep", 1., 1., 60.) as i64,
            position: saved_value(host, "clockPosition").unwrap_or_else(|| "Right".to_owned()),
            top: amount(host, "clockHeightOffset", 5., 0., 3000.) as f32,
            horizontal: amount(host, "clockWidthOffset", 0., -3000., 3000.) as f32,
            shadow: flag(host, "clockTextShadow", true),
            color: 0xFF00_0000 | (channel("clock_red") << 16)
                | (channel("clock_green") << 8) | channel("clock_blue"),
        }
    }
}

#[derive(Default)]
pub struct Clock {
    options: Mutex<Option<(u64, Options)>>,
}

fn game_time(ticks: i64, step: i64, hours24: bool) -> String {
    // Minecraft starts at 06:00; one in-game hour lasts 1000 ticks.
    let hour_ticks = (ticks.rem_euclid(24000) + 6000).rem_euclid(24000);
    let hour = hour_ticks / 1000;
    let minute = ((hour_ticks % 1000) * 60 / 1000) / step.max(1) * step.max(1);
    if hours24 { return format!("{hour:02}:{minute:02}"); }
    let label = if hour >= 12 { "PM" } else { "AM" };
    let h = if hour % 12 == 0 { 12 } else { hour % 12 };
    format!("{h}:{minute:02} {label}")
}

fn left_x(position: &str, width: f32, text_width: f32, horizontal: f32) -> f32 {
    (match position {
        "Left" => 5.,
        "Center" => (width - text_width) / 2.,
        _ => width - text_width - 5.,
    } + horizontal).max(0.)
}

impl Clock {
    pub fn render(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if event.gui_hidden || !host.playing() { return; }
        let now = host.millis();
        let options = {
            let Ok(mut lock) = self.options.lock() else { return; };
            if lock.as_ref().is_none_or(|(read, _)| now.saturating_sub(*read) >= REFRESH_MS) {
                *lock = Some((now, Options::load(host)));
            }
            let Some((_, options)) = lock.as_ref() else { return; };
            options.clone()
        };
        if !options.enabled { return; }
        let Some(player) = host.local_player() else { return; };
        if options.required && !player.inventory_slots(host).is_some_and(|items| {
            items.iter().any(|(_, stack)| stack.item().id(host) == "minecraft:clock")
        }) { return; }
        let Some(level) = player.entity().level(host) else { return; };
        let Some(ticks) = level.day_time(host) else { return; };
        let [width, height] = event.batch.size();
        if options.top + 9. > height { return; }
        let text = game_time(ticks, options.minute_step, options.hours24);
        let x = left_x(&options.position, width, event.batch.text_width(&text), options.horizontal);
        event.batch.text(&text, [x, options.top], options.color, options.shadow);
        if options.show_days && options.top + 19. <= height {
            let day = format!("Day {}", ticks.div_euclid(24000));
            let x = left_x(&options.position, width, event.batch.text_width(&day), options.horizontal);
            event.batch.text(&day, [x, options.top + 10.], options.color, options.shadow);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_hours_follow_minecraft_day() {
        assert_eq!(game_time(0, 1, true), "06:00");
        assert_eq!(game_time(6000, 1, true), "12:00");
        assert_eq!(game_time(18000, 1, true), "00:00");
        assert_eq!(game_time(0, 1, false), "6:00 AM");
        assert_eq!(game_time(12000, 15, false), "6:00 PM");
        assert_eq!(game_time(-1000, 1, true), "05:00");
    }
}
