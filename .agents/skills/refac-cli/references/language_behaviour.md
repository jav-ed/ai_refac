# Language-Specific Behaviour

Read this file when a move involves a language with non-obvious semantics or when a move is behaving unexpectedly.

## Go — whole-package moves

Moving any `.go` file cross-directory causes gopls to rename the **entire package**. All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` will cause all three to end up in the target directory. Partial-package moves are not supported.

Same-directory renames (file rename with no directory change) are a filesystem-only operation — gopls is not involved and no import paths change.

Requires `go.mod` at the project root for any cross-directory move. Without it the move will error.

## Rust — semantic module moves

Use logical module paths for any structural move:

```bash
refac move-module --project-path /path/to/cargo-workspace \
  crate::engine::matching crate::domain::matching
```

The command resolves the source with embedded rust-analyzer HIR, moves its complete file or `mod.rs` subtree, rewrites resolved references across the Cargo workspace, adjusts affected `super::` paths, creates conventional missing parent modules, and runs `cargo check --workspace --all-targets`. It never creates `#[path]` or compatibility re-export shims. A failed check rolls the planned source changes back.

Source and target must be in the same crate. A `crate::...` source that is ambiguous across workspace crates is rejected with the matching declaration locations. Workspace dependants of the selected crate are updated.

Strict v1 rejections include inline source modules, `#[path]`, attributed module declarations such as `#[cfg]`, visibility other than private/`pub`/`pub(crate)`, syntax errors, complex paths the rewriter cannot preserve, and grouped imports that would need restructuring.

Use ordinary `refac move` for same-directory `.rs` filename renames; rust-analyzer LSP rewrites the module symbol. Cross-directory `.rs` paths through `move` are rejected and direct you to `move-module`.

## Dart — package URI rewriting requires package config

`package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Without it, only relative imports are updated.

Run `dart pub get` in the project root to generate it before calling `refac`.

## TypeScript / JavaScript — tsconfig coverage

Point `--project-path` at the package containing the authoritative `tsconfig.json`. Its `include` or `files` configuration must cover all local TS/JS sources that participate in imports. External packages in `node_modules` do not need to be included.

Refac loads the complete tsconfig source set for file and directory moves while skipping recursive dependency discovery. There is no project-size path that silently omits external callers. Without tsconfig, Refac globs local source files but alias and module resolution is weaker.

### Batch safety

Each invocation has a hard limit of 30 TypeScript/JavaScript source files. Directory contents count toward the limit, and successful output reports the measured source-file count. Stop duplicate dev/build watchers first; after each batch, inspect the diff and run the build.

### Reference-update gaps

Aliases declared through `compilerOptions.paths`, including `~/*`, are rewritten for file and directory moves and checked for stale module specifiers. Aliases missing from tsconfig and arbitrary path strings, such as catalog ownership labels, cannot be mapped safely; search for old path strings and run the project build.

## Python — re-export limits

Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via that re-export, those callers are not updated.

Namespace packages (directories with no `__init__.py`) may also see incomplete updates.

## Markdown

Only relative links are rewritten. Absolute URLs and `http://` / `https://` links are left unchanged.

Links inside fenced code blocks and inline code spans are not rewritten.
