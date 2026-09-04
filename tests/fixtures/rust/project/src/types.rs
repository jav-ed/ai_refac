// Structural moves use logical module paths through `refac move-module`.

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
