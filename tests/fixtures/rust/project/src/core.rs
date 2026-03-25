use crate::types::{Config, Item, Describable};
use crate::types::Config as CoreConfig;

pub struct Engine {
    config: CoreConfig,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&self) -> String {
        self.config.describe()
    }

    pub fn process(&self, item: &Item) -> String {
        format!("{} -> {}", self.config.name, item.label)
    }
}
