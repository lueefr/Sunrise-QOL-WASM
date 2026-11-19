//! Compact Help: paginated commands visible to this player's client.
//! With no command-tree snapshot, leave command-specific usage to vanilla /help.
use crate::settings::{amount, on};
use mod_api::{
    Flow, Host, PlayerCommandEvent, PlayerCommandsSentEvent, ServerHost,
    Text, TextClick, TextStyle,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

const MAX_PLAYERS: usize = 128;
const MAX_COMMANDS: usize = 512;
const COLORS: [&str; 16] = [
    "black", "dark_blue", "dark_green", "dark_aqua",
    "dark_red", "dark_purple", "gold", "gray",
    "dark_gray", "blue", "green", "aqua",
    "red", "light_purple", "yellow", "white",
];

#[derive(Default)]
pub struct Help {
    names: Mutex<BTreeMap<u128, Vec<String>>>,
}

fn parse_help(line: &str) -> Option<Option<&str>> {
    let line = line.trim();
    let mut parts = line.splitn(2, char::is_whitespace);
    if !matches!(parts.next(), Some("help" | "?")) { return None; }
    Some(parts.next().map(str::trim).filter(|text| !text.is_empty()))
}

fn page_count(total: usize, size: usize) -> usize {
    total.saturating_add(size - 1) / size
}

fn select_page(input: Option<&str>, total: usize, size: usize) -> usize {
    input.and_then(|value| value.parse::<usize>().ok()).unwrap_or(1)
        .clamp(1, page_count(total, size).max(1))
}

fn color(host: &dyn Host, key: &str, default: f64) -> &'static str {
    let index = amount(host, key, default, 0., 15.) as usize;
    COLORS[index.min(15)]
}

fn styled(text: impl Into<String>, color: &str, click: Option<TextClick>) -> Text {
    Text::literal(text).with_style(TextStyle {
        color: Some(color.to_owned()),
        click,
        ..TextStyle::default()
    })
}

impl Help {
    pub fn sent(&self, _: &mut dyn ServerHost, event: &mut PlayerCommandsSentEvent<'_>) {
        let mut names: Vec<String> = event.names.iter().take(MAX_COMMANDS)
            .filter(|name| !name.is_empty() && name.len() <= 96
                && !name.chars().any(char::is_whitespace)).cloned().collect();
        names.sort_unstable();
        names.dedup();
        if let Ok(mut map) = self.names.lock() {
            if !map.contains_key(&event.player) && map.len() >= MAX_PLAYERS { map.clear(); }
            map.insert(event.player, names);
        }
    }

    pub fn command(&self, host: &dyn Host, event: &mut PlayerCommandEvent<'_>) -> Flow {
        if !on(host, "compact_help") { return Flow::Continue; }
        let Some(argument) = parse_help(event.command) else { return Flow::Continue; };
        // The API exposes root names, not Brigadier's argument/usage tree.
        // Let vanilla answer /help <command> instead of inventing usage hints.
        if argument.is_some_and(|value| value.parse::<usize>().is_err()) {
            return Flow::Continue;
        }
        let names = self.names.lock().ok().and_then(|map| map.get(&event.player).cloned());
        let Some(names) = names else { return Flow::Continue; };
        if names.is_empty() { return Flow::Continue; }
        let size = amount(host, "help_page_size", 8., 1., 50.) as usize;
        let command_color = color(host, "help_command_color", 2.);
        let detail_color = color(host, "help_detail_color", 7.);
        let page = select_page(argument, names.len(), size);
        let total = page_count(names.len(), size);
        let start = (page - 1) * size;

        event.replies.push(styled(
            format!("Commands — page {page}/{total}"),
            detail_color, None,
        ));
        for name in names.iter().skip(start).take(size) {
            event.replies.push(styled(
                format!("/{name}"),
                command_color,
                Some(TextClick::SuggestCommand(format!("/{name} "))),
            ));
        }
        if total > 1 {
            if page > 1 {
                event.replies.push(styled(
                    "◀ Previous page", detail_color,
                    Some(TextClick::RunCommand(format!("/help {}", page - 1))),
                ));
            }
            if page < total {
                event.replies.push(styled(
                    "Next page ▶", detail_color,
                    Some(TextClick::RunCommand(format!("/help {}", page + 1))),
                ));
            }
        }
        event.replies.push(styled(
            "Click a command to insert it into chat.", detail_color, None,
        ));
        Flow::Cancel
    }

    pub fn reset(&self) {
        if let Ok(mut map) = self.names.lock() { map.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paginated_and_bounded() {
        assert_eq!(parse_help("help"), Some(None));
        assert_eq!(parse_help("help 3"), Some(Some("3")));
        assert_eq!(parse_help("help execute as"), Some(Some("execute as")));
        assert_eq!(parse_help("heal 3"), None);
        assert_eq!(page_count(25, 8), 4);
        assert_eq!(select_page(Some("42"), 25, 8), 4);
        assert_eq!(select_page(Some("0"), 25, 8), 1);
        assert_eq!(select_page(None, 25, 8), 1);
        assert_eq!(COLORS.len(), 16);
        assert_eq!(COLORS[2], "dark_green");
        assert_eq!(COLORS[7], "gray");
    }

    #[test]
    fn text_style_preserves_click_and_color() {
        let result = styled(
            "/help", "gold",
            Some(TextClick::SuggestCommand("/help ".to_owned())),
        );
        assert_eq!(result.style.color.as_deref(), Some("gold"));
        assert_eq!(
            result.style.click,
            Some(TextClick::SuggestCommand("/help ".to_owned())),
        );
    }
}
