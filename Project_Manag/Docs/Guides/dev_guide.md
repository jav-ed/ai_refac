# Developer & Build Guide

This repo is a Rust CLI tool. The main binary is `refac`, and the supporting utility binary is `create_testbed`.

## 1. Build

Use Cargo in the normal way:

```bash
cargo build
cargo build --release
```

Main outputs:

- `target/debug/refac`
- `target/release/refac`
- `target/debug/create_testbed`

## 2. Local Availability

The expected local setup is that `refac` is reachable via `~/Progs/bin/refac`.

Preferred during development: create a symlink to the release binary.

```bash
mkdir -p ~/Progs/bin
ln -sf "$(pwd)/target/release/refac" ~/Progs/bin/refac
```

Why this is the preferred setup:

- the command path stays stable
- rebuilding `target/release/refac` updates what the symlink points to
- no extra copy step is needed after each rebuild

If `~/Progs/bin` is not already in `PATH`, add it in your shell setup.

If you need a fixed snapshot instead of a live development link, you can copy the binary instead:

```bash
mkdir -p ~/Progs/bin
cp target/release/refac ~/Progs/bin/refac
```

## 3. Main CLI

Show help:

```bash
cargo run -- --help
```

Run a move:

```bash
cargo run -- move \
  --project-path /absolute/path/to/project \
  --source-path src/old_file.ts \
  --target-path src/new_file.ts
```

Useful CLI notes:

- `--project-path` should point at the concrete package root.
- If you reuse the same root often, set `REFAC_PROJECT_PATH=/absolute/path/to/project` and omit `--project-path`.
- `--source-path` and `--target-path` are relative to `--project-path`.
- Repeat `--source-path` and `--target-path` to run a batch move.
- Add `--json` for machine-readable output.

## 4. Testbed Generator

Use `create_testbed` when you want a safe multi-language playground for manual verification:

```bash
cargo run --bin create_testbed
```

This recreates `Trials/0_Refac_Tree` and generates sample projects for:

- TypeScript
- Python
- Rust
- Go
- Dart

Each sample project has internal references so file moves can be verified against real imports or module declarations.

## 5. Basic Local Workflow

1. Build the release binary: `cargo build --release`
2. Ensure `~/Progs/bin/refac` points to it, preferably via symlink
3. Run tests when changing behavior: `cargo test`
4. Generate fresh samples if needed: `cargo run --bin create_testbed`
5. Run `refac move ...` against a concrete language project
6. Verify the affected project still builds or that its imports/modules were rewritten correctly
