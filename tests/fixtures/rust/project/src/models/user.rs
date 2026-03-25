use crate::types::{Config, Item};

pub struct UserModel {
    pub config: Config,
    pub items: Vec<Item>,
}

impl UserModel {
    pub fn new(config: Config) -> Self {
        Self { config, items: vec![] }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}
