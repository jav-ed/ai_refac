---
title: Build Guide
description: How to build refac from source and set up a local development environment.
---

## Build

Standard Cargo commands:

```bash
cargo build          # debug build
cargo build --release  # release build
```

Outputs:
- `target/debug/refac`
- `target/release/refac`
- `target/debug/create_testbed` (test fixture generator)

## Local setup

The recommended setup during development is a symlink from a stable PATH location to the release binary. Rebuilding updates the binary without any reinstall step:

```bash
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac
```

Make sure `~/.local/bin` is in your `PATH`. Most Linux distros include it automatically. If not, add to your shell config:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

If you need a fixed snapshot instead:

```bash
cp target/release/refac ~/.local/bin/refac
```

## CLI during development

Show help without installing:

```bash
cargo run -- --help
cargo run -- move --help
```

Run a move directly:

```bash
cargo run -- move \
  --project-path /absolute/path/to/project \
  --source-path src/old_file.ts \
  --target-path src/new_file.ts
```

Set `REFAC_PROJECT_PATH` to avoid repeating `--project-path`:

```bash
export REFAC_PROJECT_PATH=/absolute/path/to/project
cargo run -- move --source-path src/old.ts --target-path src/new.ts
```

## Testbed generator

`create_testbed` generates sample projects for manual verification:

```bash
cargo run --bin create_testbed
```

This recreates `Trials/0_Refac_Tree/` with sample projects for TypeScript, Python, Rust, Go, and Dart. Each project has internal references so file moves can be verified against real imports and module declarations.

## Basic development workflow

1. `cargo build --release`
2. Confirm `~/.local/bin/refac` symlink points to the new binary
3. `cargo test` after any behavior change
4. `cargo run --bin create_testbed` to regenerate samples if needed
5. Run `refac move ...` against a concrete project
6. Verify the project still builds with its native toolchain (`cargo check`, `go build ./...`, etc.)
