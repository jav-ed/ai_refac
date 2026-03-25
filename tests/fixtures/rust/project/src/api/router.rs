use crate::types::Config;
use super::api_config;

pub struct Router {
    config: Config,
}

impl Router {
    pub fn new(name: &str) -> Self {
        Self { config: api_config(name) }
    }
}
