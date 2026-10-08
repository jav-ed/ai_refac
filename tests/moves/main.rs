//! File moves through the language servers and refac's own drivers: Go, Rust, Python, Dart, and the mixed batch.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod batch;
mod dart;
mod dart_package_config;
mod go;
mod python;
mod rust;
mod rust_module;
