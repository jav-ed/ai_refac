# Install & Prerequisites

## Build from source

Requires **Rust 1.99+ (edition 2024)**. This checkout pins Rust 1.99.0 and the rustfmt/rust-analyzer components in `rust-toolchain.toml`. Install via [rustup](https://rustup.rs) if needed.

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
| Python (`rename`) | `basedpyright` (plain pyright is not enough) | `pip install basedpyright` (or `pipx`, `uv tool`) |
| Rust | `rust-analyzer` for symbol rename and ordinary file renames; semantic module support is embedded | `rustup component add rust-analyzer` (run inside the project when it pins a toolchain) |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Kotlin / Android | JetBrains Kotlin language server (`REFAC_KOTLIN_SERVER`), JDK 17+, Gradle project; `ANDROID_HOME` for Android | download and verify steps in [Kotlin server setup](../../../../Project_Manag/Docs/Setup/kotlin_Server.md) |
| Markdown | none | — |

Each of these servers is found by the same lookup (an environment variable such as `REFAC_GOPLS`, then `PATH`, then the usual install folders). `refac doctor` shows what is found on this machine, `refac doctor <language>` prints exact install steps and starts the server once to prove it works; run them instead of guessing. Details: [Language servers](../../../../Project_Manag/Docs/Setup/language_Servers.md).

Use recent external tools. The embedded rust-analyzer crates are locked with Cargo; ordinary Rust file renames use the pinned rust-analyzer component when rustup honors this checkout's toolchain file.
