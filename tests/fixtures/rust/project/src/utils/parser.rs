use super::formatter::format_config;
use crate::types::Config;

pub fn parse_config(s: &str) -> Option<Config> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 { return None; }
    parts[1].parse().ok().map(|value| Config {
        name: parts[0].to_string(),
        value,
    })
}

pub fn round_trip(c: &Config) -> Option<Config> {
    parse_config(&format_config(c).trim_matches(['[', ']']).to_string())
}
