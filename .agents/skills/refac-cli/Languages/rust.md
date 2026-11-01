# Rust

Read before the first Rust move or rename, and when one fails. Rust has two kinds of move: `move-module` takes a whole module by its logical path, and `move` renames a `.rs` file inside its folder. A Rust call takes 5 to 35 s.

## Setup

- `--project-path`: the folder with `Cargo.toml` (package or workspace).
- Tool: `rust-analyzer`, for symbol rename and ordinary file renames; the semantic module support of `move-module` is embedded. Install it with `rustup component add rust-analyzer` (run inside the project when it pins a toolchain).
- `refac doctor rust` finds it (`REFAC_RUST_ANALYZER` overrides the lookup) and proves it starts.

## Move a module: `move-module`

Use logical module paths for any structural move:

```bash
refac move-module --project-path /path/to/cargo-workspace \
  crate::engine::matching crate::domain::matching
```

The command resolves the source with embedded rust-analyzer HIR, moves its complete file or `mod.rs` subtree, rewrites resolved references across the Cargo workspace, adjusts affected `super::` paths, creates conventional missing parent modules, and runs `cargo check --workspace --all-targets`. It never creates `#[path]` or compatibility re-export shims. A failed check rolls the planned source changes back.

Source and target must be in the same crate. A `crate::...` source that is ambiguous across workspace crates is rejected with the matching declaration locations. Workspace dependants of the selected crate are updated.

Strict v1 rejections include inline source modules, `#[path]`, attributed module declarations such as `#[cfg]`, visibility other than private/`pub`/`pub(crate)`, syntax errors, complex paths the rewriter cannot preserve, and grouped imports that would need restructuring.

## Rename a file in its folder: `move`

Use ordinary `refac move` for same-directory `.rs` filename renames; rust-analyzer LSP rewrites the module symbol. Cross-directory `.rs` paths through `move` are rejected and direct you to `move-module`.

## Rename a symbol

`refac rename` follows traits, `use` trees, and re-exports. Names written inside `macro_rules!` are not renamed; the output starts with `ATTENTION` and names the lines to edit by hand. A `mod` name is refused: use `refac move-module`. Several renames of one project go into one `rename --batch` call (measured on the Rust fixture: four renames take 18.8 s as four commands and 4.6 s as one batch, same files).
