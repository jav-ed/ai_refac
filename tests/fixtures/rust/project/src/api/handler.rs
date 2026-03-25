use crate::types::{Config, Item};

pub fn handle_config(c: &Config) -> String {
    format!("handled: {}", c.name)
}

pub fn handle_item(i: &Item) -> String {
    format!("item: {}", i.label)
}
