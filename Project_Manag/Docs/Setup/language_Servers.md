# Language servers

Every semantic operation of refac (a symbol rename, and the file moves of Go, Rust, Dart, and Kotlin) is done by a language server that someone else wrote. refac does not ship them and does not download them. It **finds** one, **starts** it for the command, **stops** it when the command ends, and **explains** exactly what to do when it cannot find one. The explanation is the product: an agent that hits a missing server should be able to fix it alone.

## One command to ask

```bash
refac doctor            # one line per language: ready, MISSING, or BROKEN; starts nothing
refac doctor go         # details for one language, plus a start-and-stop check of the server
refac doctor rust --json   # for scripts; exit code 1 when the server is not ready
```

`refac doctor <language>` prints, for that language: what the server is used for, where it was found (or every place that was looked at and what was there), the version, the tools it needs (`go` for gopls, `cargo` for rust-analyzer), a start check (it starts the server on an empty folder, waits for the answer to `initialize`, stops it), and numbered install steps that can be pasted. Names it accepts: `go`, `rust`, `python`, `dart`, `kotlin`, `typescript` (and `golang`, `rs`, `py`, `flutter`, `kt`, `android`, `ts`, `javascript`, `js`). `--project-path` matters only where the project decides: rustup picks the Rust toolchain from the project, and Python looks in the project's `.venv`.

When a command that needs a server cannot find one, its own error already carries the search record and the pointer:

```text
The Go language server (gopls) was not found.
refac starts it by itself for each command and stops it afterwards, so no server has to be running; it has to be installed. Looked here:
  - $REFAC_GOPLS: not set
  - PATH: no command named `gopls`
  - /root/go/bin/gopls: no such file
Run `refac doctor go` to see how to install or fix it, then repeat the command.
```

The same text is printed by `refac rename`, by `refac move` on a Go, Rust, Dart, or Kotlin batch, and by the real-server tests when they are run without the server.

## Where a server is looked for

The order is fixed, and every place is recorded:

1. The language's **environment variable**. A value that is set but wrong is the answer ("`$REFAC_GOPLS` is set to /no/such/gopls: cannot run"); there is no fallback behind it, because falling back would hide the mistake.
2. **`PATH`**.
3. **The folders installers use**, in a fixed list per language.

A place counts only when the server there runs: refac executes its version command (20 second limit). That matters for rustup, which puts a `rust-analyzer` stand-in in `~/.cargo/bin` even when the component is not installed; the stand-in exists and fails with "Unknown binary 'rust-analyzer'", and refac reports that as a broken install instead of using it.

| Language | Server | Variable | Also looked for in | Install |
| :--- | :--- | :--- | :--- | :--- |
| Go | gopls | `REFAC_GOPLS` | `$GOBIN`, `$GOPATH/bin`, `~/go/bin` | `go install golang.org/x/tools/gopls@latest` |
| Rust | rust-analyzer | `REFAC_RUST_ANALYZER` | `$CARGO_HOME/bin`, `~/.cargo/bin` | `rustup component add rust-analyzer` (inside the project when it pins a toolchain) |
| Python | basedpyright | `REFAC_PYTHON_SERVER` | the project's `.venv/bin` and `venv/bin`, `~/.local/bin` | `pip install basedpyright` (or `pipx`, `uv tool`) |
| Dart | the SDK's `dart language-server` | `REFAC_DART` | `$FLUTTER_ROOT/bin/cache/dart-sdk/bin` | Dart SDK from dart.dev; Flutter ships its own |
| Kotlin | JetBrains Kotlin language server | `REFAC_KOTLIN_SERVER` (the unpacked folder) | nowhere | download and checksum, see [Kotlin server setup](kotlin_Server.md) |
| TypeScript / JavaScript | TypeScript 7 native engine, bundled | none | installed by refac on first use | only Bun has to be present |

The install steps live in one file per server in [`src/servers/catalog/`](../../../src/servers/catalog/); update them there when an installer changes, and `refac doctor` and every error follow.

## Started for the command, stopped after it

A language server idle in the background holds hundreds of megabytes, and an agent runs refac a handful of times per task. So nothing stays resident: the server starts when the command needs it, loads the project, answers, and is shut down (and whatever it created is deleted) before the files are written. Measured on the test fixtures (a few files each, debug build, one run after another on a quiet machine); larger projects take longer in proportion to what the server must load:

| Language | `rename --dry-run` end to end | Largest process |
| :--- | :--- | :--- |
| Go (gopls) | 1.7 s | 165 MB |
| Rust (rust-analyzer) | 4.6 s | 637 MB |
| Python (basedpyright) | 1.3 s | 158 MB |
| Dart (analysis server) | 0.4 s | 124 MB |
| Kotlin (JetBrains server) | about 38 s | about 1.6 GiB, plus the Gradle daemon |

The Kotlin cost is the Gradle import; see [Kotlin server setup](kotlin_Server.md). A limit on how long a server may take is `REFAC_LSP_TIMEOUT_SECS` for the others and `REFAC_KOTLIN_TIMEOUT_SECS` for Kotlin; a server that does not answer in time ends the command with an error that names the variable. `REFAC_LSP_TRACE=1` prints every message exchanged with a server (`full` keeps them whole), which is how a misbehaving server is investigated.

## Checking a machine

`refac doctor` is the quick check. The tests that need a server are `#[ignore]`d and print the same explanation when it is missing, so `cargo test --test go_rename -- --ignored` on a machine without gopls fails with the install steps instead of a cryptic error. `tests/doctor.rs` runs the real binary with an empty `PATH` and an empty `HOME` and needs nothing installed.

Python file moves use other tools (Rope, Pyrefly); `refac move` on a Python batch with neither says so. `refac doctor python` is about the rename server.
