---
title: Testing & Debugging
description: How to run the test suite and debug failures.
---

## Run all tests

```bash
cargo test
```

## Run a specific test

```bash
cargo test drivers::rust::tests::test_rust_cross_dir_move_keeps_project_buildable -- --nocapture
```

Run all tests for a single language:

```bash
cargo test go_move
cargo test dart_move
```

## Test coverage

The suite covers:

- Shared LSP edit application
- CLI help and validation
- TypeScript, Python, Rust, Go, Dart, and Markdown move flows
- Batch moves across all languages, including partial failure and cross-package Go batches

## Integration test architecture

Each language has a fixture directory and a test file:

| Language | Fixture | Test file | Move under test |
|---|---|---|---|
| TypeScript | `tests/fixtures/typescript/project/` | `tests/typescript_move.rs` | `src/models/User.ts` → `src/core/User.ts` |
| Python | `tests/fixtures/python/project/` | `tests/python_move.rs` | `myapp/utils/formatters.py` → `myapp/core/formatters.py` |
| Rust | `tests/fixtures/rust/project/` | `tests/rust_move.rs` | `src/types.rs` → `src/shared/types.rs` |
| Go | `tests/fixtures/go/project/` | `tests/go_move.rs` | `pkg/utils/format.go` → `pkg/helpers/format.go` |
| Dart | `tests/fixtures/dart/project/` | `tests/dart_move.rs` | `lib/src/formatter.dart` → `lib/src/core/formatter.dart` |
| Markdown | `tests/fixtures/markdown/` | `tests/markdown_move.rs` | various `.md` link rewrites |

**Fixtures are never modified.** Each test copies the fixture into a temp directory before running. The tool operates on the copy; originals stay pristine and are cleaned up automatically.

## Batch tests

`tests/batch_move.rs` exercises multi-file invocations using the real CLI binary:

| Scenario | What is tested |
|---|---|
| TypeScript: unrelated pair | Two files with no import relationship |
| TypeScript: cross-importing pair | Both files move to the same dir; import between them is rewritten |
| Python: two files with cross-import | Sequential Rope moves; second move sees the already-updated import |
| Rust: two same-dir files | Single rust-analyzer session; project still compiles after batch |
| Go: same-package batch | Both files from one package — gopls moves the whole package once |
| Go: cross-package batch | Files from two different packages — one gopls session for both |
| Dart: two files | Cross-import between them is rewritten |
| Markdown: two files | Links between them are rewritten |
| Mixed language (TS + Markdown) | Two languages dispatched independently in one CLI call |
| Partial failure | One language succeeds, one fails — both reported |
| All fail | Exit non-zero with structured error message |

## Implementation notes

**Go temp directories:** The test helper uses `refac-test-` as a temp dir prefix (not a hidden `.tmp` prefix). gopls skips workspace roots whose directory starts with `.`, so hidden temp dirs prevent import cascade in Go tests.

**Dart serialization:** The Dart analysis server is sensitive to concurrent starts. `dart_move.rs` acquires a global `Mutex` before each test so at most one analysis server runs at a time.

**External tooling:** Tests that require external tools (gopls, rust-analyzer, etc.) skip gracefully if the tool is not installed — they do not fail, but they also do not provide coverage.

## Adding a fixture

Drop a new file in the fixture directory, then add assertions in the corresponding `*_move.rs`. Follow the existing pattern: snapshot control files before the move, assert changes after, use comments to document limitations.

## Manual CLI verification

Generate sample projects:

```bash
cargo run --bin create_testbed
```

Run the CLI against one:

```bash
./target/debug/refac move \
  --project-path Trials/0_Refac_Tree/typescript \
  --source-path src/models/TaskManager.ts \
  --target-path src/core/TaskManager.ts
```

## Post-move validation

After a move, verify with the native toolchain:

| Language | Command |
|---|---|
| Rust | `cargo check` |
| Go | `go build ./...` |
| TypeScript | run the project typecheck/build |
| Python | import the affected modules or run project tests |
| Dart | run the package analyzer/build |

## Common debugging

**TypeScript is slow or hanging:** `--project-path` is too broad. Point it at the specific sub-package with `tsconfig.json`, not a monorepo root.

**"Source path does not exist":** `--source-path` is resolved relative to `--project-path`. If the file exists but the tool can't find it, the project path is wrong.

**Backend not running:** Each language requires its external tooling. Check that `bun`, `rope`, `rust-analyzer`, `gopls`, or `dart` is installed and in PATH.
