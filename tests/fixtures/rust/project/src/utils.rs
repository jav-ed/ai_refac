use crate::types::Config;

pub mod formatter;
pub mod parser;
pub mod validator;

pub fn default_config(name: &str) -> Config {
    Config { name: name.to_string(), value: 0 }
}
