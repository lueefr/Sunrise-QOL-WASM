//! World-local first-join markers; no filesystem access outside the mod's WASM services.
use mod_api::{ConfigFileError, ConfigPath, PlayerLoggedInEvent, ServerHost, Text};
use crate::settings::on;

pub fn on_join(host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
    let welcome = on(host, "welcome_message");
    let first = on(host, "first_join_message");
    if !welcome && !first { return; }
    let mut first_time = false;
    if first {
        if let Ok(path) = ConfigPath::new(&format!("first_join/{:032x}.seen", event.player)) {
            let files = host.world_data_files();
            if matches!(files.read(&path), Err(ConfigFileError::NotFound)) {
                first_time = files.write(&path, b"1").is_ok();
            }
        }
    }
    let Some(player) = host.player(event.player) else { return; };
    if first_time {
        let _ = player.tell(host, Text::literal("You wake up in an unfamiliar place."));
    }
    if welcome {
        let _ = player.tell(host, Text::literal("Welcome to the server!"));
    }
}
