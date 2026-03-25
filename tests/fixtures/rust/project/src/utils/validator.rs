use crate::types::{Config, Item, Describable};

pub fn validate_config(c: &Config) -> bool {
    !c.name.is_empty()
}

pub fn validate_item(i: &Item) -> bool {
    i.id > 0 && !i.label.is_empty()
}

pub fn validated_description<T: Describable>(t: &T) -> Option<String> {
    Some(t.describe())
}
