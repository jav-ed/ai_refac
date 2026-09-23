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

Refac scans the complete tsconfig source set with Oxc and resolves code imports with TypeScript without constructing a type checker. Batches remain limited to 30 contained source files. Ordinary apply/verification failures roll back; a crash or forced termination can interrupt rollback.

The helper is terminated and reaped after 5 minutes or sampled RSS above 4 GiB (100 ms sampling). `REFAC_TYPESCRIPT_MAX_RSS_MB` changes the threshold in positive integer MiB. Inspect the working tree after a limit failure. One-file batches still scan every configured source; do not shrink coverage to hide callers.

### Reference-update gaps

Declared aliases are rewritten and verified. Project references, symlink moves, overlapping requests, and ambiguous locally bound `require` calls fail explicitly. Computed imports, arbitrary path strings, comments, and package/config metadata need manual audit. Search old paths and run the project build. See [TypeScript backend](../../../../Project_Manag/Docs/Features/TypeScript/linker_TypeScript.md) for supported forms and exact limits.

## Python — re-export limits

Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via that re-export, those callers are not updated.

Namespace packages (directories with no `__init__.py`) may also see incomplete updates.

## Markdown

Only relative links are rewritten. Absolute URLs and `http://` / `https://` links are left unchanged.

Links inside fenced code blocks and inline code spans are not rewritten.
