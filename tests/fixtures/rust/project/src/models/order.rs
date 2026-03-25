// Aliased use — `Item as OrderItem`.
use crate::types::Item as OrderItem;

pub struct Order {
    pub id: u64,
    pub items: Vec<OrderItem>,
}

impl Order {
    pub fn new(id: u64) -> Self {
        Self { id, items: vec![] }
    }

    pub fn total_items(&self) -> usize {
        self.items.len()
    }
}
