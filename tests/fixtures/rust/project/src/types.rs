// MOVE TARGET: src/types.rs -> src/shared/types.rs
//
// The Rust driver uses a SHIM strategy for cross-directory moves:
// - This file is physically moved to src/shared/types.rs
// - src/shared/mod.rs is created with `mod types;` + `pub use crate::types;`
// - src/lib.rs `pub mod types;` is patched with `#[path = "shared/types.rs"]`
// - All caller files are left UNCHANGED (they compile via the alias)

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u64,
    pub label: String,
}

pub trait Describable {
    fn describe(&self) -> String;
}

impl Describable for Config {
    fn describe(&self) -> String {
        format!("Config({}: {})", self.name, self.value)
    }
}

impl Describable for Item {
    fn describe(&self) -> String {
        format!("Item({}: {})", self.id, self.label)
    }
}
