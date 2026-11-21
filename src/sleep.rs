//! Sleep Sooner changes the clock to dusk before vanilla checks the bed.
use crate::settings::{amount, on};
use mod_api::{Flow, Hand, RightClickBlockEvent, ServerHost};

pub fn click(host: &mut dyn ServerHost, event: &RightClickBlockEvent<'_>) -> Flow {
    if event.click.hand != Hand::Main || !on(host, "sleep_sooner") { return Flow::Continue; }
    let Some(player) = host.players().acting_player() else { return Flow::Continue; };
    let Some(level) = player.level(host) else { return Flow::Continue; };
    if !level.in_place(host) { return Flow::Continue; }
    let Some(block) = level.block(host, event.click.pos) else { return Flow::Continue; };
    if !block.id(host).ends_with("_bed") { return Flow::Continue; }
    let Some(time) = level.clock_time(host) else { return Flow::Continue; };
    let time_of_day = time.rem_euclid(24000);
    let start = amount(host, "sleep_start", 12000., 0., 24000.) as i64;
    if start > 12540 && time_of_day > 12540 && time_of_day < start { return Flow::Cancel; }
    if time_of_day >= start && time_of_day <= 12540 {
        let _ = level.set_clock_time(host, time - time_of_day + 12540);
    }
    Flow::Continue
}
