//! Symbol rename against the real language servers (gopls, rust-analyzer, basedpyright, the Dart SDK), the batch, and wide characters with CRLF.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod batch;
mod dart;
mod dry_run_batch;
mod encoding;
mod go;
mod python;
mod rust;
