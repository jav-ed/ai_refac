# Testing & Debugging Guide

This guide is for the current CLI workflow.

## 1. Automated Tests

Run the full suite:

```bash
cargo test
```

Run a targeted test:

```bash
cargo test drivers::rust::tests::test_rust_cross_dir_move_keeps_project_buildable -- --nocapture
```

The current test suite covers:

- shared LSP edit application
- CLI help and validation
- TypeScript move flow
- Python move flow
- Rust same-dir and cross-dir move flow
- Go move flow, including cross-dir filename changes
- Dart move flow

## 2. Integration Test Architecture

Each language has a fixture directory and a test file:

| Language   | Fixture                          | Test file                    | Move under test                                       |
|------------|----------------------------------|------------------------------|-------------------------------------------------------|
| TypeScript | `tests/fixtures/typescript/project/` | `tests/typescript_move.rs` | `src/models/User.ts` → `src/core/User.ts`            |
| Python     | `tests/fixtures/python/project/`     | `tests/python_move.rs`     | `src/models/user.py` → `src/core/user.py`            |
| Rust       | `tests/fixtures/rust/project/`       | `tests/rust_move.rs`       | `src/types.rs` → `src/shared/types.rs`               |
| Go         | `tests/fixtures/go/project/`         | `tests/go_move.rs`         | `pkg/utils/format.go` → `pkg/helpers/format.go`      |
| Dart       | `tests/fixtures/dart/project/`       | `tests/dart_move.rs`       | `lib/src/formatter.dart` → `lib/src/core/formatter.dart` |

**Fixtures are never modified by running tests.** `common::setup_fixture` copies the fixture into a `tempfile::TempDir` before each test. The tool operates on the temp copy; the originals stay pristine and the temp dir is cleaned up automatically when the test ends. No reset step is needed.

Run a single language's tests:

```bash
cargo test go_move
cargo test dart_move
```

**To add a new fixture file:** drop it in the fixture directory, then add assertions in the corresponding `*_move.rs`. Follow the pattern in the file: snapshot control files before the move, assert positive changes after, use comments to document limitations explicitly.

## 3. Manual CLI Verification

Generate sample projects:

```bash
cargo run --bin create_testbed
```

Then run the CLI against one concrete package root:

```bash
./target/debug/refac move \
  --project-path Trials/0_Refac_Tree/typescript \
  --source-path src/models/TaskManager.ts \
  --target-path src/core/TaskManager.ts
```

For machine-readable output:

```bash
./target/debug/refac move \
  --json \
  --project-path Trials/0_Refac_Tree/go \
  --source-path pkg/service/ledger.go \
  --target-path pkg/ledger/ledger.go
```

## 4. Project-Level Validation

After a move, validate the affected project with its native toolchain when possible:

- Rust: `cargo check`
- Go: `go build ./...`
- Python: import the affected modules or run project tests
- TypeScript: run the package typecheck/build if available
- Dart: run the package analyzer/build if available

## 5. Debugging Notes

### TypeScript scans are too broad

If TypeScript work becomes slow or memory-heavy, the usual problem is an overly broad `--project-path`.

- Good: point `--project-path` at the concrete TypeScript package that owns `tsconfig.json`
- Bad: point `--project-path` at a monorepo root and pass long nested paths

### Source path does not exist

`--source-path` is resolved relative to `--project-path`. If the file is real but the tool cannot find it, the root is usually wrong.

### Backend-specific tooling is missing

Each language backend depends on external tooling:

- TypeScript: `bun`
- Python: `python3` plus configured Python refactor backend
- Rust: `rust-analyzer`
- Go: `gopls`
- Dart: `dart`
