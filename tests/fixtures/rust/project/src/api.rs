use crate::types::Config;

pub mod handler;
pub mod router;

pub fn api_config(name: &str) -> Config {
    Config { name: name.to_string(), value: 42 }
}
