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

## 2. Manual CLI Verification

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

## 3. Project-Level Validation

After a move, validate the affected project with its native toolchain when possible:

- Rust: `cargo check`
- Go: `go build ./...`
- Python: import the affected modules or run project tests
- TypeScript: run the package typecheck/build if available
- Dart: run the package analyzer/build if available

## 4. Debugging Notes

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
