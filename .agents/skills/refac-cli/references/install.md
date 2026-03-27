# Install & Prerequisites

## Build from source

Requires **Rust 1.85+ (edition 2024)**. Install via [rustup](https://rustup.rs) if needed.

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
| Rust | `rust-analyzer` | [rust-analyzer.github.io](https://rust-analyzer.github.io) |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Markdown | none | — |

Use recent releases. Older versions of gopls and rust-analyzer may behave differently or fail silently.
