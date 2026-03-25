use super::default_config;
use crate::types::{Config, Item};

pub fn format_config(c: &Config) -> String {
    format!("[{}={}]", c.name, c.value)
}

pub fn format_item(i: &Item) -> String {
    format!("<{}: {}>", i.id, i.label)
}

pub fn describe_default(name: &str) -> String {
    format_config(&default_config(name))
}
