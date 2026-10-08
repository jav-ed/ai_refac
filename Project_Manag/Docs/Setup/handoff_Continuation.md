# Handoff: where the work stands

Written on 2026-10-07 at the end of the session that added the Kotlin backend, rebuilt the shared language-server client, audited every tool version, rebuilt Markdown moves, and added symbol rename for Go, Rust, Python, and Dart with `refac doctor`. Everything below is pushed on the branch `claude/quirky-darwin-dh1zd2` (draft pull request into `main`; the Markdown and symbol-rename work of the later rounds is committed on the same branch). The working notes of that session live in the gitignored `Scratch/` folder and are not needed to continue.

## State of the code

- **TypeScript symbol rename** (`refac rename`): done and tested. Overview in [Symbol rename](../Features/TypeScript/symbol_Rename.md), evidence in [TypeScript rename engines](../Investigation/typescript_Rename_Engines.md).
- **Kotlin backend** (file and directory moves, `refac rename`, the Android XML and `R`/`BuildConfig` layer, stale-name report, rollback): done and tested against the real JetBrains server. Overview in [Kotlin and Android](../Features/Kotlin/linker_Kotlin.md), setup in [Kotlin server setup](kotlin_Server.md), evidence in [Kotlin options](../Investigation/kotlin_Options.md).
- **Symbol rename in Go, Rust, Python, and Dart**, and Kotlin ported onto the same engine (`src/drivers/lsp/rename/`): gopls, rust-analyzer, basedpyright, and the Dart analysis server, with an in-memory proof, override families for Python, retries for gopls, and a settle step for Dart. Overview in [Symbol rename](../Features/Symbol_Rename/linker_Symbol_Rename.md), evidence in [Symbol rename options](../Investigation/symbol_Rename_Options.md). Real-server tests: `tests/{go,rust,python,dart}_rename.rs`, all `#[ignore]`d.
- **Batch rename** (`refac rename --batch`, `src/drivers/lsp/rename/batch.rs`, `src/cli/rename.rs`): several renames of one project and language in one server session, all or nothing, for Kotlin, Go, Rust, Python, and Dart. A keep-alive server was considered and rejected; the reasons and numbers are at the end of [Symbol rename options](../Investigation/symbol_Rename_Options.md). Real-server tests: `tests/rename/batch.rs` and two batch scenarios in `tests/kotlin/rename.rs`.
- **`refac doctor` and the server locator** (`src/servers/`, `src/cli/doctor.rs`): every language server is found the same way (variable, `PATH`, installer folders, each candidate must run) and a missing one is explained with the places looked at and `Run refac doctor <language>`. Servers are started per command and stopped after it. See [Language servers](language_Servers.md).
- **Shared language-server client** (`src/drivers/lsp/client.rs` on `src/drivers/lsp/session.rs`): used by Dart, Go, Rust, and Pyrefly. It no longer sleeps for a fixed time; each server's own readiness signal is awaited and a missing signal fails after `REFAC_LSP_TIMEOUT_SECS` (default 300). The Dart, Go, batch, Rust, and Python suites pass on a machine under heavy load. Dart checks the server's plan before writing and refuses a move that would leave an import pointing at a missing file.
- **Markdown**: the link scanner is now a CommonMark parser (`pulldown-cmark`), so links in code, comments, raw HTML, and front matter stay untouched, and a relative `--project-path` rewrites links like an absolute one. See [Limits and gaps](../Features/Markdown/limits_And_Gaps.md).
- **Versions**: all tools were audited on 2026-10-07 and brought to their latest releases where the whole suite stayed green; see [Tool versions](tool_Versions.md) for the table, the pins, and the alternatives that were weighed.

## What is not in git

| Item | Where | How to get it |
|---|---|---|
| Kotlin language server `ILS-263.6379.0` | wherever `REFAC_KOTLIN_SERVER` points | The URL and sha256 in [Kotlin server setup](kotlin_Server.md). refac never downloads it. Its bundled early-access key is valid through 2026-10-30; after that a newer build is needed. |
| Android SDK (platform 36, build-tools 36.0.0) | `ANDROID_HOME` | Only the Android tests need it; they panic with an explanation when it is missing. |
| Reference clones | `Repos/` (gitignored) | Commands in [External reference repos](repos_List.md) |
| `scripts/node_modules` | gitignored | `bun install --frozen-lockfile` inside `scripts/` (the Rust code also runs it when missing) |
| Public docs site | sibling repo `../Refac_Docs` | Its TypeScript, usage, capabilities, and index pages were edited locally by an earlier session but never committed, and it holds unrelated uncommitted changes, so the owner decides what is pushed. Its Rust, Go, and Python pages were stale at that time, and it was not checked from the machine of this session whether anything there describes the Kotlin backend or the later work. |
| Release binary | `~/.local/bin/refac` on the owner's machine | Not rebuilt here. Run `cargo build --release` after pulling, as the [docs entry point](../doc_Start.md) says. |

## Known limits and untested corners

- Symbol rename: Python calls on untyped receivers, Rust `macro_rules!` bodies, code behind inactive `cfg` or build constraints, and generated files are reported, not renamed. Package, module, and file names are `move` operations. gopls can answer with part of the edits under heavy load (the engine asks up to four times). Dart needs `dart pub get`. The real-server suites for Go, Rust, Python, and Dart were run on this session's tool versions (see [Tool versions](tool_Versions.md)); they are not part of plain `cargo test`.
- Checked on real code (x/tools, this repository, Rope, the Dart `collection` package), in workspaces, behind symlinks and hidden folders, and with wide characters and CRLF: see the end of [Symbol rename options](../Investigation/symbol_Rename_Options.md). [`ci_workflow.yml`](ci_workflow.yml) is the CI definition (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the plain `cargo test`). It sits here and not in `.github/workflows/` because the GitHub App that pushes for these sessions has no `workflows` permission and the push is refused; an owner activates it with `mkdir -p .github/workflows && cp Project_Manag/Docs/Setup/ci_workflow.yml .github/workflows/ci.yml`. It was written without being able to run GitHub Actions from here, so read its first run. The real-server suites are `#[ignore]`d, need the four servers installed, and are run by hand.
- Batch rename does not cover TypeScript/JavaScript (its own engine, one rename per command), mixes of languages, or more than one project per batch.
- Ideas not built: `refac move --dry-run`, Markdown heading rename, a Python rename that starts from an override and reaches the base.

- Kotlin: moves between modules or source sets are refused, so cross-module moves are untested. Every Kotlin call costs about 30 seconds of Gradle import and about 1.6 GiB for the server on a tiny project; the server has no memory cap. One broken-build test hung for more than five minutes in one of nine runs and could not be reproduced; the 600 second timeout and the import log tail are the guard.
- Tests that need a language server (gopls, Dart SDK, rope, basedpyright, Kotlin server) are `#[ignore = "needs X (run `refac doctor Y`)"]` and fail loudly when run without the tool; a plain `cargo test` needs only Rust (+ Bun) and no test skips silently.
- Pyrefly is a fallback behind Rope, so `python_move` never starts it; `cargo test --lib pyrefly -- --ignored` runs it for real.
- A blanket `cargo update` can break the build (the `ra-ap-rustc_lexer` Unicode check); build after every update and read [Tool versions](tool_Versions.md).

## First commands

```bash
cargo build
cargo test
cd scripts && bun install --frozen-lockfile && bun test
```

For the symbol-rename suites install gopls, rust-analyzer, basedpyright, and the Dart SDK (`refac doctor` shows what is missing) and run `cargo test --test rename -- --include-ignored --test-threads=1` (the group `tests/rename/`: Go, Rust, Python, Dart, encoding and batch). For the real-server Kotlin suites set `REFAC_KOTLIN_SERVER` and `ANDROID_HOME`, then `cargo test --test kotlin -- --ignored --test-threads=1` (the group `tests/kotlin/`).
