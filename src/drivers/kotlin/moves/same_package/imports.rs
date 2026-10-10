//! What a file imports, to see which names are already taken.

use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Imports {
    bound: HashSet<String>,
    stars: HashSet<String>,
}

impl Imports {
    pub fn parse(text: &str) -> Self {
        let mut imports = Self::default();
        for line in text.lines() {
            let Some(path) = line.trim().strip_prefix("import ") else {
                continue;
            };
            let path = path.split("//").next().unwrap_or(path);
            let path = path.trim().trim_end_matches(';').trim();
            if let Some((_, alias)) = path.split_once(" as ") {
                imports
                    .bound
                    .insert(alias.trim().trim_matches('`').to_string());
            } else if let Some(package) = path.strip_suffix(".*") {
                imports.stars.insert(package.trim().to_string());
            } else if let Some(name) = path.rsplit('.').next() {
                imports.bound.insert(name.trim_matches('`').to_string());
            }
        }
        imports
    }

    /// Some import already gives this simple name a meaning (whatever the
    /// package), so a second one would clash with it.
    pub fn binds(&self, name: &str) -> bool {
        self.bound.contains(name)
    }

    pub fn star(&self, package: &str) -> bool {
        self.stars.contains(package)
    }
}
