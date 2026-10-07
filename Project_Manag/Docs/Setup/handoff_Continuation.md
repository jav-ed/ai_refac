# Handoff: where the work stopped

Written on 2026-10-07 when the previous session was asked to stop and push so another person (or session) can continue from a clean checkout. The previous session's working notes live in the gitignored `Scratch/` folder on its own machine and are deliberately not pushed; ask that session for the task list and the Kotlin probe harness, it hands them over directly.

## State of the code

- **Done and tested: TypeScript symbol rename** (`refac rename`). Overview in [Symbol rename](../Features/TypeScript/symbol_Rename.md), evidence in [TypeScript rename engines](../Investigation/typescript_Rename_Engines.md). Rust code lives in `src/drivers/typescript/rename*` and `src/logic/rename.rs`, tests in `tests/typescript_rename.rs` (19 tests), fixture in `tests/fixtures/typescript/rename_project/`.
- **Dependencies were updated** (Bun: `oxc-parser` 0.153, `typescript` 6.0.3, plus the `typescript-native` alias for TypeScript 7.0.2; Cargo: 13 selective updates). A blanket `cargo update` breaks the build against the pinned `ra_ap_*` crates, so never run it. Revert option: `git checkout <base> -- Cargo.lock`.
- **Done and tested against the real server: the Kotlin backend** (moves, directory moves, `refac rename`, Android XML and `R`/`BuildConfig` layer, stale-name report, rollback). Overview in [Kotlin and Android](../Features/Kotlin/linker_Kotlin.md), setup in [Kotlin server setup](kotlin_Server.md), evidence in [Kotlin options](../Investigation/kotlin_Options.md). Code in `src/drivers/kotlin/` and `src/logic/route.rs`; tests in `tests/kotlin_*.rs` (`#[ignore]`d, they need the server) with fixtures in `tests/fixtures/kotlin/`.
- **Known failing tests, not caused by this work:** the Dart suites (`cargo test --lib dart`, `cargo test --test dart_move`) fail when the machine is busy, because `src/drivers/lsp_client.rs` waits fixed sleeps and then reports success without rewriting imports. All other targets pass. Details in the task list.

## Where the plan is

- The full task list and the ordered next steps for Kotlin are in the previous session's local `Scratch/Agent_Tasks/ts_variable_rename_24a2a9b7.md` (not in git). In short: fix the readiness wait in the probe harness (wait for the server's import-finished notifications, not for idle progress), run the file and package move scenarios, run the rename scenarios, build an Android fixture, record results in [Kotlin options](../Investigation/kotlin_Options.md), then implement the Kotlin backend with tests and docs.
- Follow the house rules in `.agents/skills/coding/SKILL.md` (files of at most 300 lines, no fallbacks: fail loudly) and create your own task file in `Scratch/Agent_Tasks/` as the `temp-task-file` skill describes.

## What is not in git

| Item | Where | How to get it |
|---|---|---|
| Kotlin language server `ILS-263.6379.0` | `/home/jav/Progs/kotlin-lsp/kotlin-server-263.6379.0/` | `https://download.jetbrains.com/language-server/kotlin-server/263.6379.0/kotlin-server-263.6379.0.tar.gz`, 368,488,700 bytes, sha256 `ab8ca4455dc2fc5fe1a24db2bccc46c104254d2c465155c4251ee65df8f3f7cc`. Needs the owner's permission to download again. Its bundled EAP key is valid through 2026-10-30. |
| Kotlin probe harness and fixture | `Scratch/Kotlin_Lsp_Trial/` on the previous session's machine | Ask the previous session; it is throw-away code (a small Python LSP client plus a Gradle fixture) |
| Reference clones | `Repos/` (gitignored) | Commands in [External reference repos](repos_List.md) |
| `scripts/node_modules` | gitignored | `bun install --frozen-lockfile` inside `scripts/` (the Rust code also runs it when missing) |
| Public docs edits | sibling repo `../Refac_Docs` | The TypeScript, usage, capabilities and index pages were updated locally there but not committed; that repo also holds unrelated uncommitted changes, so the owner decides what gets pushed. Its Rust, Go and Python pages are still stale. |

## Tools on the machine used so far

Rust 1.98.1, Bun 1.4.2, JDK 24, Gradle 8.14.3, Python 3.13, Android SDK at `/home/jav/Progs/Android`. Software is installed under `/home/jav/Progs/<tool>/`.

## First commands

```bash
cargo build
cargo test --test typescript_rename
cd scripts && bun install --frozen-lockfile && bun test
```
