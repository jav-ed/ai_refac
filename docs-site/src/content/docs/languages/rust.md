---
title: Rust
description: How refac handles Rust file moves using rust-analyzer.
---

The Rust backend uses [rust-analyzer](https://rust-analyzer.github.io/) via LSP. It handles two structurally different cases — same-directory renames and cross-directory moves — with different strategies for each.

## Required tooling

`rust-analyzer` — checked at `rust-analyzer` in PATH, then `~/.cargo/bin/rust-analyzer`.

## Same-directory rename

When source and target share the same directory, the filename change corresponds to a module rename. The driver:

1. Locates the `mod <name>;` declaration in the parent module file (`src/lib.rs`, `src/main.rs`, or the directory's `mod.rs`).
2. Issues a `textDocument/rename` on the module name symbol. rust-analyzer rewrites all `use` paths that reference it across the crate.
3. Moves the file on the filesystem.

**Result:** All `use crate::...` paths to this module are fully updated everywhere in the crate.

## Cross-directory move (shim strategy)

Cross-directory moves require changing the module tree, which LSP rename cannot do safely. The driver uses a static rewrite instead:

1. Finds the existing `mod <name>;` declaration in the declaring file.
2. Moves the source file to the target path.
3. Rewrites the declaration to add a `#[path = "..."]` attribute pointing at the new location, preserving the original module name so all existing `use crate::...` paths continue to compile.
4. Adds a `pub use crate::<old_path>` alias in the target directory's module file so external callers using the old path reach the module through a re-export.

**This is a shim strategy.** Callers are not rewritten — they continue to use the old path through the alias. The project stays buildable, but import paths are not fully migrated. The `#[path]` attribute and `pub use` alias will appear in your diff as permanent code changes.

## Batch session architecture

When a batch contains multiple same-directory renames, all are processed in **one rust-analyzer session**. Each rename fires sequentially within that session with `textDocument/didChange` notifications between them. One startup regardless of batch size.

Cross-directory moves are handled before the LSP batch, without a language server (static file rewrites only).

## Limitations

**Cross-directory moves do not rewrite caller imports.** The shim keeps the project buildable but leaves old `use` paths in caller files intact. This is a known gap — full cross-directory migration via LSP is not implemented.

**Single crate only.** The driver operates on one Cargo crate at a time. Workspace-wide cross-crate reference updates are not supported.

**`mod.rs` moves** are handled via `willRenameFiles`, not symbol rename, and are treated as a separate case.
