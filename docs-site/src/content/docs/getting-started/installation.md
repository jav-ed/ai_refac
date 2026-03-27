---
title: Installation
description: How to build and install refac on Linux or macOS.
---

## Prerequisites

**Rust 1.85+ (edition 2024)** is required to build `refac`. Install via [rustup](https://rustup.rs) if needed.

Each language backend also requires its own external tooling:

| Language | Required | Install |
|---|---|---|
| TypeScript / JS | `bun` | [bun.sh](https://bun.sh) |
| Python | `rope` importable from `.venv` or `python3` | `pip install rope` |
| Python (fallback) | `pyrefly` (only if Rope is absent) | `pip install pyrefly` |
| Rust | `rust-analyzer` | [rust-analyzer.github.io](https://rust-analyzer.github.io) |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Markdown | none | — |

You only need the tooling for the languages you intend to use.

## Build from source

```bash
git clone https://github.com/jav-ed/ai_refac.git
cd ai_refac
cargo build --release
```

## Add to PATH

Choose one of the following:

```bash
# Symlink — rebuilding updates it automatically (recommended during development)
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac

# Fixed snapshot copy
cp target/release/refac ~/.local/bin/refac

# Install via cargo
cargo install --path .
```

Make sure `~/.local/bin` (or wherever you placed the binary) is in your `PATH`.

## Platform support

Linux and macOS. Windows is untested and not supported.
