# Developer & Build Guide

This repo is a Rust CLI tool. The main binary is `refac`, and the supporting utility binary is `create_testbed`.

The crate requires Rust 1.99 or newer. `rust-toolchain.toml` pins development to Rust 1.99.0 with rustfmt and rust-analyzer.

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

The expected local setup is that `refac` is reachable via `~/.local/bin/refac`.

Preferred during development: `scripts/install.sh`. It builds the release binary, creates the symlink, and verifies the result (see section 6).

```bash
scripts/install.sh
```

By hand the same is a symlink to the release binary:

```bash
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac
```

Why this is the preferred setup:

- the command path stays stable
- rebuilding `target/release/refac` updates what the symlink points to
- no extra copy step is needed after each rebuild

If `~/.local/bin` is not already in `PATH`, add it in your shell setup.

If you need a fixed snapshot instead of a live development link, you can copy the binary instead:

```bash
mkdir -p ~/.local/bin
cp target/release/refac ~/.local/bin/refac
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

Run a semantic Rust module move:

```bash
cargo run -- move-module \
  --project-path /absolute/path/to/cargo-workspace \
  crate::engine::matching \
  crate::domain::matching
```

Useful CLI notes:

- For `move`, `--project-path` should point at the concrete package root.
- For Rust `move-module`, it may point at a Cargo package or workspace root.
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

1. Install the release binary: `scripts/install.sh` (builds, links, verifies)
2. If you only built it by hand, ensure `~/.local/bin/refac` points to it, preferably via symlink
3. Run tests when changing behavior: `cargo test`
4. Generate fresh samples if needed: `cargo run --bin create_testbed`
5. Run `refac move ...` against a concrete language project
6. Verify the affected project still builds or that its imports/modules were rewritten correctly

## 6. Keep the Global Install Current

**Run `scripts/install.sh` after every pull, checkout or source change.** It is one command and idempotent: it runs `cargo build --release` (seconds when nothing changed), points `~/.local/bin/refac` at `target/release/refac`, and then checks three things, stopping with exit code 1 and the reason when one fails:

1. a `refac` is found on the PATH (otherwise `~/.local/bin` is missing from the PATH);
2. that `refac` is this checkout's `target/release/refac` and not another copy that shadows it;
3. `refac --version` names the commit that is checked out.

```text
installed and current: refac 0.1.4 (55b157a, built 2026-10-09 20:09 UTC)
```

The stamp comes from `build.rs` and exists in the release profile only, so debug and test builds do not recompile after every commit (they print `dev build`). `+dirty` after the commit means a file under `src/`, `Cargo.toml`, `Cargo.lock` or `build.rs` differs from that commit (the helper scripts in `scripts/` are read from the checkout at run time, not compiled in). To check without building, run `refac --version` and compare it with `git rev-parse --short HEAD`.

The install needs only the release build: `target/release` is about 1.5 GB. The much larger debug and test programs are a by-product of `cargo test`; keep them in check as described in [Resource use](../Setup/resource_Use.md).
