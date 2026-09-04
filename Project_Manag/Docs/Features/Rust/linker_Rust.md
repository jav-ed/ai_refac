# Rust

The Rust backend keeps simple file renames and structural module moves separate. Same-directory `.rs` renames use the rust-analyzer language server. `move-module` uses embedded rust-analyzer HIR to move one complete logical module subtree and rewrite resolved references without `#[path]` or compatibility shims.

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
4. finds resolved references across workspace crates and plans their new paths;
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
- Existing source visibility is preserved for the moved declaration and generated parent declarations.

No `#[path]` attribute or old-path re-export is introduced. Callers migrate to the new path.

## Workspace references

References in the selected crate and dependent workspace crates are rewritten from rust-analyzer's resolved reference graph. For example, moving `crate::matching` to `crate::domain::matching` in package `core_lib` also changes `core_lib::matching::value` in a dependent workspace package to `core_lib::domain::matching::value`.

## Strict v1 limits

Refac stops before mutation when it encounters a layout it cannot preserve confidently. Current hard errors include:

- inline source modules;
- `#[path]` modules;
- attributed module declarations, including `#[cfg]`;
- visibility other than private, `pub`, or `pub(crate)`;
- an inline descendant that declares an out-of-line child;
- Rust syntax errors in a file that must be rewritten;
- complex path syntax or a grouped import that would require structural rewriting;
- a target that already exists or a target inside the source subtree;
- resolved source references outside the selected Cargo workspace.

Proc-macro expansion and build-script output loading are disabled for v1. The final Cargo check remains authoritative and triggers rollback if the moved workspace does not compile.

## Ordinary Rust file moves

Same-directory file renames use one rust-analyzer LSP session for the batch and rename the module symbol before the filesystem move. A cross-directory `.rs` path passed to ordinary `refac move` fails with guidance to use `move-module`; Refac does not silently create a shim.

The external `rust-analyzer` binary is therefore required for ordinary file renames. `move-module` uses the embedded rust-analyzer libraries locked in `Cargo.lock`.
