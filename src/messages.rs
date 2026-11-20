//! First Join Message and Welcome Message share one player-login hook.
//! Each module owns its own settings and the first-join marker is world-local.
use crate::settings::{flag, on, saved_value};
use mod_api::{
    ConfigFileError, ConfigPath, PlayerLoggedInEvent, ServerHost, ServerStartedEvent,
    Text,
};
use mod_api::text::TextClick;
use std::sync::atomic::{AtomicBool, Ordering};

static DEDICATED: AtomicBool = AtomicBool::new(false);

const COLORS: [&str; 16] = [
    "black", "dark_blue", "dark_green", "dark_aqua",
    "dark_red", "dark_purple", "gold", "gray",
    "dark_gray", "blue", "green", "aqua",
    "red", "light_purple", "yellow", "white",
];

fn color(host: &dyn ServerHost, key: &str, default: usize) -> Option<&'static str> {
    let index = saved_value(host, key)
        .map(|value| value.parse::<usize>().ok())
        .unwrap_or(Some(default))?;
    COLORS.get(index).copied()
}

fn styled(message: String, color: &str, link: &str) -> Text {
    let mut text = Text::literal(message);
    text.style.color = Some(color.to_owned());
    let link = link.trim();
    if link.starts_with("https://") || link.starts_with("http://") {
        text.style.click = Some(TextClick::OpenUrl(link.to_owned()));
    }
    text
}

fn template(message: &str, name: &str) -> String {
    message.replace("%s", name)
}

pub fn started(event: &ServerStartedEvent) {
    DEDICATED.store(event.dedicated, Ordering::Relaxed);
}

pub fn reset() {
    DEDICATED.store(false, Ordering::Relaxed);
}

fn first_join(host: &mut dyn ServerHost, event: &PlayerLoggedInEvent<'_>) {
    if !on(host, "first_join_message") { return; }
    let Ok(path) = ConfigPath::new(&format!("first_join/{:032x}.seen", event.player))
        else { return; };
    let files = host.world_data_files();
    if !matches!(files.read(&path), Err(ConfigFileError::NotFound)) { return; }
    if files.write(&path, b"1").is_err() { return; }

    let message = saved_value(host, "firstJoinMessage")
        .unwrap_or_else(|| "You wake up in an unfamiliar place.".to_owned());
    if !message.is_empty() {
        if let (Some(player), Some(color)) = (
            host.player(event.player), color(host, "firstJoinMessageTextFormattingColourIndex", 2)
        ) {
            let _ = player.tell(host, styled(template(&message, event.name), color, ""));
        }
    }

    let broadcast = saved_value(host, "serverBroadcastMessage").unwrap_or_default();
    if !broadcast.is_empty() {
        if let Some(color) = color(host, "serverBroadcastMessageTextFormattingColourIndex", 3) {
            let _ = host.messages().broadcast(styled(template(&broadcast, event.name), color, ""));
        }
    }
}

fn welcome(host: &mut dyn ServerHost, event: &PlayerLoggedInEvent<'_>) {
    if !on(host, "welcome_message") { return; }
    if flag(host, "onlyRunOnDedicatedServers", true)
        && !DEDICATED.load(Ordering::Relaxed) {
        return;
    }
    let Some(player) = host.player(event.player) else { return; };
    let configs = [
        ("messageOneText", "Welcome to the server!", "messageOneColourIndex", 2,
         "messageOneOptionalURL"),
        ("messageTwoText", "Downloaded from CurseForge! This is a clickable link.",
         "messageTwoColourIndex", 14, "messageTwoOptionalURL"),
        ("messageThreeText", "You should probably edit this in the config :)",
         "messageThreeColourIndex", 15, "messageThreeOptionalURL"),
    ];
    let mut blank = flag(host, "sendEmptyLineBeforeFirstMessage", true);
    for (key, fallback, color_key, index, url_key) in configs {
        let text = saved_value(host, key).unwrap_or_else(|| fallback.to_owned());
        if text.is_empty() { continue; }
        let Some(color) = color(host, color_key, index) else { return; };
        if blank {
            let _ = player.tell(host, Text::literal(""));
            blank = false;
        }
        let url = saved_value(host, url_key).unwrap_or_else(|| {
            if url_key == "messageTwoOptionalURL" {
                "https://curseforge.com/members/serilum/projects".to_owned()
            } else { String::new() }
        });
        let _ = player.tell(host, styled(text, color, &url));
    }
}

pub fn on_join(host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
    first_join(host, event);
    welcome(host, event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_colors_follow_original_ids() {
        assert_eq!(COLORS.len(), 16);
        assert_eq!(COLORS[2], "dark_green");
        assert_eq!(COLORS[14], "yellow");
        assert_eq!(COLORS[15], "white");
    }

    #[test]
    fn name_template_and_links() {
        assert_eq!(template("Welcome %s, %s", "Alex"), "Welcome Alex, Alex");
        let link = styled("Visit".to_owned(), "green", "https://example.org");
        assert_eq!(link.style.color.as_deref(), Some("green"));
        assert_eq!(link.style.click, Some(TextClick::OpenUrl("https://example.org".into())));
        assert_eq!(styled("No link".into(), "red", "file:///etc/passwd").style.click, None);
    }
}
