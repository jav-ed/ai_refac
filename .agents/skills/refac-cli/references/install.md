# Install & Prerequisites

## Build from source

Requires **Rust 1.98+ (edition 2024)**. This checkout pins Rust 1.98.1 and the rustfmt/rust-analyzer components in `rust-toolchain.toml`. Install via [rustup](https://rustup.rs) if needed.

```bash
git clone https://github.com/jav-ed/ai_refac.git
cd ai_refac
cargo build --release
```

## Add to PATH

```bash
# symlink — rebuilding updates it automatically (recommended)
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac

# or copy a fixed snapshot
cp target/release/refac ~/.local/bin/refac

# or install from the local checkout
cargo install --path .
```

**Platform:** Linux and macOS. Windows is untested and not supported.

## Language backend prerequisites

Each language requires its own external tooling. Only install what you need.

| Language | Required | Install |
|---|---|---|
| TypeScript / JS | `bun` | [bun.sh](https://bun.sh) |
| Python | `rope` importable from `.venv` or `python3` | `pip install rope` |
| Python (fallback) | `pyrefly` (only if Rope is absent) | `pip install pyrefly` |
| Rust | `rust-analyzer` for ordinary file renames; semantic module support is embedded | [rust-analyzer.github.io](https://rust-analyzer.github.io) |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Markdown | none | — |

Use recent external tools. The embedded rust-analyzer crates are locked with Cargo; ordinary Rust file renames use the pinned rust-analyzer component when rustup honors this checkout's toolchain file.
