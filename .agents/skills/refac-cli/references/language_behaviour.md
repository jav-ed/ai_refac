# Language-Specific Behaviour

Read this file when a move involves a language with non-obvious semantics or when a move is behaving unexpectedly.

## Go — whole-package moves

Moving any `.go` file cross-directory causes gopls to rename the **entire package**. All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` will cause all three to end up in the target directory. Partial-package moves are not supported.

Same-directory renames (file rename with no directory change) are a filesystem-only operation — gopls is not involved and no import paths change.

Requires `go.mod` at the project root for any cross-directory move. Without it the move will error.

## Rust — cross-directory moves use a shim

Moving a Rust file to a different directory does **not** rewrite caller imports. Instead it:
1. Adds a `#[path = "..."]` attribute in the declaring file pointing to the new location.
2. Adds a `pub use crate::...` alias so existing callers continue to compile.

These are permanent code changes that will appear in your diff. Caller files are not migrated — they keep working through the alias. To fully migrate callers you must update them manually or run a follow-up rename.

Same-directory renames (file rename within the same directory) do fully rewrite all `use` paths via rust-analyzer.

Single crate only — cross-crate reference updates are not supported.

## Dart — package URI rewriting requires package config

`package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Without it, only relative imports are updated.

Run `dart pub get` in the project root to generate it before calling `refac`.

## TypeScript / JavaScript — large project threshold

For individual file moves in projects with more than ~500 TS/JS source files, `refac` skips loading the full project. Only the moved file's own imports are rewritten — files that import it are not updated.

The 500-file threshold excludes `node_modules`, `dist`, `build`, `.next`, and `.git`.

To stay under the threshold, point `--project-path` at the sub-package root rather than the monorepo root.

Directory moves always load the full project regardless of size and may be slow on large codebases.

## Python — re-export limits

Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via that re-export, those callers are not updated.

Namespace packages (directories with no `__init__.py`) may also see incomplete updates.

## Markdown

Only relative links are rewritten. Absolute URLs and `http://` / `https://` links are left unchanged.

Links inside fenced code blocks and inline code spans are not rewritten.
