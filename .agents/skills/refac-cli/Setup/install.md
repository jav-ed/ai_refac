# Install and prerequisites

Read this file when `refac` itself is not installed or not on the PATH. The tools of a language (its server, its `--project-path` and how to install it) are in the Setup section of that language's file in `Languages/`; open only the language you work in. `refac doctor` shows what is found on this machine and `refac doctor <language>` prints exact install steps, so run them instead of guessing.

## Build from source

Requires **Rust 1.99+ (edition 2024)**. This checkout pins Rust 1.99.0 and the rustfmt/rust-analyzer components in `rust-toolchain.toml`. Install via [rustup](https://rustup.rs) if needed.

```bash
git clone https://github.com/jav-ed/ai_refac.git
cd ai_refac
scripts/install.sh
```

`scripts/install.sh` builds the release binary (a 36 MB file; the 1.5 GB around it in `target/release` is build cache that keeps repeat builds fast), links it as `~/.local/bin/refac`, and checks that the `refac` on the PATH is that build and comes from the checked-out commit. Run it again after every pull; `refac --version` shows the commit the installed binary was built from (`+dirty` means local changes).

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

## Language tools

Each language requires its own external tooling. Only install what you need: the tools for a language you never move or rename in stay uninstalled, and `refac doctor` showing them as `MISSING` needs no action. The Setup section of each file in `Languages/` names the tool, its install command and the right `--project-path`; the Markdown file needs nothing.

Each of these servers is found by the same lookup (an environment variable such as `REFAC_GOPLS`, then `PATH`, then the usual install folders). `refac doctor <language>` also starts the server once to prove it works; `refac guide servers` lists the environment variables and limits.

Use recent external tools. The embedded rust-analyzer crates are locked with Cargo; ordinary Rust file renames use the pinned rust-analyzer component when rustup honors this checkout's toolchain file.
