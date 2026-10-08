# Rust

The Rust backend keeps simple file renames, structural module moves, and symbol renames separate. `refac rename` on an `.rs` file renames a function, method, field, variant, type, or local with rust-analyzer ([Rust symbol rename](../Symbol_Rename/rust.md); a `mod` name is not a symbol rename, it is `move-module`). The server is found by the shared locator (`REFAC_RUST_ANALYZER`, `PATH`, `~/.cargo/bin`) and `refac doctor rust` explains a missing one. Same-directory `.rs` renames use the rust-analyzer language server. `move-module` uses embedded rust-analyzer HIR to move one complete logical module subtree and rewrite resolved references without `#[path]` or compatibility shims.

## Commands

Rename a module file without changing its parent module:

```bash
refac move --project-path /path/to/package \
  --source-path src/domain/old_name.rs \
  --target-path src/domain/new_name.rs
```

Move a logical module to a different parent:

```bash
refac move-module --project-path /path/to/cargo-workspace \
  crate::engine::matching \
  crate::domain::matching
```

Structural source and target arguments always begin with `crate::`. They identify modules in the same crate, not filesystem paths. If the source path matches multiple workspace crates, Refac stops and reports the matching declaration files.

## Semantic move sequence

Before mutation, Refac:

1. loads the Cargo package or workspace with embedded rust-analyzer crates;
2. resolves the source as an out-of-line HIR module and confirms that the target does not exist;
3. discovers its conventional physical representation: `name.rs` plus an optional `name/` companion, or the complete `name/mod.rs` directory;
4. finds resolved references across workspace crates, including code behind `#[cfg(test)]` and the arguments of macro calls, and plans their new paths;
5. plans declaration removal/insertion, missing conventional parent modules, and `super::` rewrites inside moved files;
6. rejects target collisions, overlapping edits, and unsupported source layouts.

It then applies the plan, reloads the workspace to confirm the new logical path resolves and the old one does not, and runs:

```bash
cargo check --workspace --all-targets
```

If semantic reload or Cargo validation fails, Refac reverses the physical moves and restores every planned source-file write.

## Conventional layout

The semantic command moves the module's complete representation:

- `src/engine/matching.rs` moves as one module file; `src/engine/matching/`, when present, moves with it.
- `src/engine/matching/mod.rs` moves with the entire `src/engine/matching/` directory.
- Missing target parents are represented conventionally with `mod.rs` and a matching `mod <name>;` declaration.
- The visibility of the declaration keeps its reach. `pub` and `pub(crate)` are copied; a private `mod` (and `pub(self)`, `pub(super)`) that lands in a deeper parent is written `pub(super)` or `pub(in crate::…)` so that everything that named the module before still can. The same holds for generated parent declarations.
- The new `mod` line joins the first block of declarations at the top of its parent file, in alphabetical order when that block is sorted, not after a `mod tests;` at the end.

No `#[path]` attribute or old-path re-export is introduced. Callers migrate to the new path.

## Workspace references

References in the selected crate and dependent workspace crates are rewritten from rust-analyzer's resolved reference graph. For example, moving `crate::matching` to `crate::domain::matching` in package `core_lib` also changes `core_lib::matching::value` in a dependent workspace package to `core_lib::domain::matching::value`.

## What the rewrite covers

Every reference is rewritten from rust-analyzer's resolved graph, so the form in the source decides what is written:

- `crate::old::path` and `super::` paths become the new absolute path; a leading `super::` inside the moved file that leaves the moved module becomes `crate::…` (and `pub(super)` becomes `pub(in crate::parent)`), while a `super::` that stays inside the module is left alone.
- A name imported with `use` stays short: with `use super::matching;` in place, `matching::value()` keeps its form and only the import is rewritten. A local variable, constant or function that merely carries the module's name is not a reference and is not touched.
- A module that leaves the group of an import (`use super::{helper, matching};`) is taken out of the group and imported on the next line, with its alias or nested list; a pair loses its braces, a group of one becomes a plain import.
- A path in the arguments of a macro call (`vec![old::Item { .. }]`, `assert_eq!(super::old::value(), 7)`) is a token, not a path, and rust-analyzer lists it only when it can expand the macro (it cannot expand the standard library's `assert_eq!` and `vec!`). Refac reads the tokens itself: every `word::word::…` chain is cut at the module's name and rust-analyzer's name resolution, asked in the scope of the call, decides whether it is the moved module. So `crate::`, `super::`, `self::`, imported and nested spellings, `$crate::` in a `macro_rules!` body and a dependent crate's `crate_name::old::…` are all rewritten like a path outside a macro, and a different module of the same name is left alone. In the moved file, a `super::` in macro arguments that leaves the module becomes `crate::…` too (not inside a `macro_rules!` definition, where `super` means the call site).
- A path inside generic arguments (`Vec<old::Item>`) is a path of its own.

## Strict v1 limits

Refac stops before mutation when it encounters a layout it cannot preserve confidently. Current hard errors include:

- inline source modules;
- `#[path]` modules;
- module declarations with an attribute other than a condition or a lint (`cfg`, `allow`, `warn`, `deny`, `forbid`, `expect`, `doc`, `deprecated`); `#[path]` and `#[macro_use]` are refused by name;
- `pub(in …)` visibility other than the absolute `pub(in crate::…)`;
- an inline descendant that declares an out-of-line child;
- Rust syntax errors in a file that must be rewritten;
- complex path syntax in a reference (a module path with generic arguments of its own, for instance), and an import with attributes that would have to leave its group;
- a target that already exists or a target inside the source subtree;
- resolved source references outside the selected Cargo workspace.

Proc-macro expansion and build-script output loading are disabled for v1. The final Cargo check remains authoritative and triggers rollback if the moved workspace does not compile.

## Ordinary Rust file moves

Same-directory file renames use one rust-analyzer LSP session for the batch and rename the module symbol before the filesystem move. A cross-directory `.rs` path passed to ordinary `refac move` fails with guidance to use `move-module`; Refac does not silently create a shim.

The external `rust-analyzer` binary is therefore required for ordinary file renames. `move-module` uses the embedded rust-analyzer libraries locked in `Cargo.lock`.

## Evidence: moving this repository's own modules

The `src/drivers/` tree of this repository was reorganised with `refac move-module` itself (`lsp_client`, `lsp_session`, `lsp_rename` and its stages, `symbol_*`, the Kotlin Android modules, the Rust driver's own helpers); each move took about 4.5 minutes, most of it the final `cargo check --workspace --all-targets`. Every case where the tool first failed became a regression test in `tests/moves/rust_module/` (and unit tests under `src/drivers/rust/`): generic type paths, code behind `#[cfg(test)]`, a `super::*` inside the moved module, a `cfg` declaration and a `cfg(test)` child, short references through an import, paths inside macro arguments (first `crate::` only, later every spelling: `super::`, `self::`, imported, `$crate::`, a dependent crate's name), module paths inside generic arguments, a local variable with the module's name, `pub(super)`, a private module moved deeper, and a module leaving a grouped import.
