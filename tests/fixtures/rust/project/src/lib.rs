// This fixture exercises ordinary Rust file renames and rejection of structural
// file moves through `refac move`.
pub mod types;
pub mod error;
pub mod config;
pub mod utils;
pub mod models;
pub mod services;
pub mod core;
pub mod api;
pub mod prelude;

pub use types::Config;
pub use types::Item;
