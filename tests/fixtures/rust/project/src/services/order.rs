// Glob use — `use crate::types::*`
use crate::types::*;

pub struct OrderService;

impl OrderService {
    pub fn make_item(id: u64, label: &str) -> Item {
        Item { id, label: label.to_string() }
    }

    pub fn describe_config(c: &Config) -> String {
        c.describe()
    }
}
