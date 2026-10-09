# Install and prerequisites

Read this file when `refac` is not installed or not on the PATH, or when a language backend has to be set up. `refac doctor` shows what is found on this machine and `refac doctor <language>` prints exact install steps, so run them instead of guessing.

## Build from source

Requires **Rust 1.99+ (edition 2024)**. This checkout pins Rust 1.99.0 and the rustfmt/rust-analyzer components in `rust-toolchain.toml`. Install via [rustup](https://rustup.rs) if needed.

```bash
git clone https://github.com/jav-ed/ai_refac.git
cd ai_refac
scripts/install.sh
```

`scripts/install.sh` builds the release binary (about 1.5 GB in `target/`), links it as `~/.local/bin/refac`, and checks that the `refac` on the PATH is that build and comes from the checked-out commit. Run it again after every pull; `refac --version` shows the commit the installed binary was built from (`+dirty` means local changes).

## Add to PATH by hand

The script already links the binary. Without it:

```bash
# symlink: rebuilding updates it automatically (recommended)
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac

# or copy a fixed snapshot
cp target/release/refac ~/.local/bin/refac

# or install from the local checkout
cargo install --path .
```

**Platform:** Linux. macOS and Windows are not targets, so nothing is tested or promised for them.

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
| Kotlin / Android | JetBrains Kotlin language server (`REFAC_KOTLIN_SERVER` names its folder), JDK 17+, Gradle project; `ANDROID_HOME` for Android | `refac doctor kotlin` prints the download and verify steps (refac never downloads it) |
| Markdown | none | nothing to install |

Each of these servers is found by the same lookup (an environment variable such as `REFAC_GOPLS`, then `PATH`, then the usual install folders). `refac doctor <language>` also starts the server once to prove it works; `refac guide servers` lists the environment variables and limits.

Use recent external tools. The embedded rust-analyzer crates are locked with Cargo; ordinary Rust file renames use the pinned rust-analyzer component when rustup honors this checkout's toolchain file.
