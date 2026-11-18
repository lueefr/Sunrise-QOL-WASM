//! Compact Help: paginated commands visible to this player's client.
//! With no command-tree snapshot, leave vanilla /help alone instead of guessing.
use crate::settings::{amount, on};
use mod_api::{Flow, Host, PlayerCommandEvent, PlayerCommandsSentEvent, ServerHost, Text};
use std::collections::BTreeMap;
use std::sync::Mutex;

const MAX_PLAYERS: usize = 128;
const MAX_COMMANDS: usize = 512;

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

impl Help {
    pub fn sent(&self, _: &mut dyn ServerHost, event: &mut PlayerCommandsSentEvent<'_>) {
        let mut names: Vec<String> = event.names.iter().take(MAX_COMMANDS)
            .filter(|name| !name.is_empty() && name.len() <= 96).cloned().collect();
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
        let names = self.names.lock().ok().and_then(|map| map.get(&event.player).cloned());
        let Some(names) = names else { return Flow::Continue; };
        if names.is_empty() { return Flow::Continue; }
        let size = amount(host, "help_page_size", 8., 1., 50.) as usize;
        if let Some(command) = argument.filter(|value| value.parse::<usize>().is_err()) {
            let name = command.trim_start_matches('/');
            if !names.iter().any(|entry| entry == name) {
                return Flow::Continue;
            }
            event.replies.push(Text::literal(format!(
                "/{name} — for command-specific usage, run /minecraft:help {name}"
            )));
            return Flow::Cancel;
        }
        let page = select_page(argument, names.len(), size);
        let start = (page - 1) * size;
        event.replies.push(Text::literal(format!(
            "Commands — page {page}/{}",
            page_count(names.len(), size)
        )));
        for name in names.iter().skip(start).take(size) {
            event.replies.push(Text::literal(format!("/{name}")));
        }
        event.replies.push(Text::literal(format!(
            "Use /help <page> to see another page ({}/{})", page, page_count(names.len(), size)
        )));
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
        assert_eq!(parse_help("heal 3"), None);
        assert_eq!(page_count(25, 8), 4);
        assert_eq!(select_page(Some("42"), 25, 8), 4);
        assert_eq!(select_page(Some("0"), 25, 8), 1);
        assert_eq!(select_page(None, 25, 8), 1);
    }
}
