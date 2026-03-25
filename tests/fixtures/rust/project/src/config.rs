use crate::types::Config;

pub fn default_config() -> Config {
    Config { name: "default".to_string(), value: 0 }
}

pub fn named_config(name: &str, value: u32) -> Config {
    Config { name: name.to_string(), value }
}
