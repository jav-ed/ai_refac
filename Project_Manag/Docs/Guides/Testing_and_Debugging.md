# Testing & Debugging Guide

This guide is for the current CLI workflow.

## 1. Automated Tests

Run the full suite:

```bash
cargo test
bun install --cwd scripts --frozen-lockfile
bun run --cwd scripts test
bun run --cwd scripts typecheck
```

Run a targeted test:

```bash
cargo test drivers::rust::tests::test_rust_cross_dir_move_keeps_project_buildable -- --nocapture
```

The current test suite covers:

- shared LSP edit application
- CLI help and validation
- TypeScript CLI moves, parser/resolver edge cases, rollback, 3,005-file batches under a 1 GiB RSS budget, and helper termination
- Python move flow (Rope backend)
- Rust same-dir and cross-dir move flow
- Go move flow, including whole-package rename cascade
- Dart move flow, including the refusal of a plan that would leave `package:` imports dangling
- Markdown move flow
- The shared rename engine (`src/drivers/lsp/rename/`): the proof, related groups, override families, planning, retries, and each language's rules, all against a fake server that only knows words; the server locator (`src/servers/`) against fake executables
- `refac doctor` and the missing-server messages, on a machine with no server at all (`tests/cli/doctor.rs`)
- `rename --batch`: the batch engine against the word server (`lsp/rename/batch/tests.rs`), the Kotlin change notifications (`kotlin/resync/tests.rs`), the dispatch (`logic/rename/tests.rs`), and the command line (`tests/cli/batch_rename.rs`)
- Symbol rename in Go, Rust, Python, and Dart against the real servers (`#[ignore]`d, section 2, Rename tests)
- Kotlin and Android: pure planning, XML rewriting, import insertion, and verification logic in the normal suite; the scenarios against the real Kotlin server are `#[ignore]`d (section 2, Kotlin tests)
- Batch moves across all languages, including partial failure and cross-package Go batches

## 2. Integration Test Architecture

### Layout: six test programs

Every file directly in `tests/` would be its own program linking the whole library (rust-analyzer included), about 90 MB each, so `tests/` holds **groups**, one program per folder (`tests/<group>/main.rs`, with the files of the group as modules). **Do not add a `.rs` file directly in `tests/`**; add a module to a group, and see [resource use](../Setup/resource_Use.md) for the numbers.

| Group (`--test`) | Holds | Real server needed |
|---|---|---|
| `cli` | usage errors, `--help`, `refac guide`, `doctor`, `rename --batch` refusals, the answer and exit code of a partly failed `move`, and `layout` (the shape of the code: see below) | no |
| `moves` | `move` per language (Go, Rust, Python, Dart), `move-module`, multi-language batches | partly: the `go`, `dart` and `python` tests are `#[ignore]`d (`-- --ignored` with the tool installed) |
| `rename` | symbol rename in Go, Rust, Python, Dart, encoding, batch | yes, `#[ignore]`d |
| `typescript` | moves, rename, limits, the 3,005-file stress test | no (the TypeScript engine is installed by refac) |
| `markdown` | link rewriting, sites, corpus, scale, safety | no |
| `kotlin` | server, moves, rename, dispatch, Android | yes, `#[ignore]`d |

Run one group with `cargo test --test <group>`, a part of it with a module filter (`cargo test --test rename go::`), the real-server ones with `-- --ignored`. A header comment in each file says the same.

The CI definition ([`ci_workflow.yml`](../Setup/ci_workflow.yml), activated by copying it to `.github/workflows/ci.yml`, see the [handoff](../Setup/handoff_Continuation.md)) runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the plain `cargo test`, so the same three commands are the check before a push. The `#[ignore]`d tests need real servers and run by hand.

### Layout: the shape of the code is a test

`tests/cli/layout.rs` fails the run when the code drifts from the rules of the file-tree and coding skills:

- no `.rs` file directly in `tests/` (each one is a 90 MB program);
- every code file is declared by its parent module (a file nobody declares is never compiled, so its tests never run);
- at most 300 lines of code per file (blank lines and comments do not count), so a long test file becomes a file of helpers plus one child module per subject (`tests/moves/rust_module.rs` with `rust_module/{basic,paths,imports,declarations}.rs` is the pattern);
- at most 9 code files directly in one folder, so related files get a folder (`src/drivers/rust/` has `analysis/`, `edits/` and `transaction/`).

The reorganisation of `src/drivers/` was done with `refac move-module` itself; see the evidence section of [Rust](../Features/Rust/linker_Rust.md).

### Per-language tests

Each language has a fixture directory and a test file:

| Language   | Fixture                              | Test file                    | Move under test                                          |
|------------|--------------------------------------|------------------------------|----------------------------------------------------------|
| TypeScript | `tests/fixtures/typescript/project/` | `tests/typescript/moves.rs`   | `src/models/User.ts` → `src/core/User.ts`               |
| TypeScript rename | `tests/fixtures/typescript/rename_project/` | `tests/typescript/rename.rs` | symbol renames, clash and shadow refusals, UTF-8/BOM/CRLF, config rejection |
| Python     | `tests/fixtures/python/project/`     | `tests/moves/python.rs`       | `myapp/utils/formatters.py` → `myapp/core/formatters.py`|
| Rust       | `tests/fixtures/rust/project/`       | `tests/moves/rust.rs`         | `src/types.rs` → `src/shared/types.rs`                  |
| Go         | `tests/fixtures/go/project/`         | `tests/moves/go.rs`           | `pkg/utils/format.go` → `pkg/helpers/format.go`         |
| Dart       | `tests/fixtures/dart/project/`       | `tests/moves/dart.rs`, `tests/moves/dart/package_config.rs` | `lib/src/formatter.dart` → `lib/src/core/formatter.dart`, and the refusal without `package_config.json` |
| Markdown   | `tests/fixtures/markdown/`           | `tests/markdown/moves.rs`     | various `.md` link rewrites                             |
| Move dry run | the move fixtures, plus scratch projects for TypeScript and Markdown | `tests/moves/dry_run/` (`typescript`, `markdown`, `refusals` need Bun only; `dart`, `go`, `python`, `rust` are `#[ignore]`d), `scripts/Tests/TypeScript/dry_run.test.ts` (the TypeScript resolver check against the files after the move agrees with the real move), `tests/moves/rust_module/dry_run.rs` (`--check`), `src/drivers/preview/copy/tests.rs` (the copy-and-compare engine, no tool) | for each backend: the dry run changes no file, then the real move edits exactly the files the plan named and moves the paths it listed (`tests/common/dry_run.rs::assert_plan_matches_move`); refusals match the real move; a Cargo.lock written while loading is removed |
| Go rename | `tests/fixtures/go/rename_module/` | `tests/rename/go.rs` | interface family, clash refusal, package clause refusal, dry run, `go build` + `go vet` after each rename |
| Rust rename | `tests/fixtures/rust/rename_crate/` | `tests/rename/rust.rs` | trait methods, shorthand fields, enum variants, `macro_rules!` note, module refusal, `cargo check` after each rename |
| Python rename | `tests/fixtures/python/rename_project/` | `tests/rename/python.rs` | override family, `__all__`, keyword arguments, untyped receiver note, module refusal; the program and its checks are run after each rename |
| Dart rename | `tests/fixtures/dart/rename_package/` | `tests/rename/dart.rs` | overrides, `export show`, field formals, named parameters, missing `package_config.json`; `dart analyze --fatal-infos` after each rename |
| Encoding | the four rename fixtures | `tests/rename/encoding.rs` | CRLF files and wide characters before the symbol on the same line, in Go, Rust, Python, and Dart; the project builds, the text and every line ending are intact |
| Batch rename | the four rename fixtures | `tests/rename/batch.rs` (real servers), `tests/cli/batch_rename.rs` (no server) | four renames in one session, one by the name an earlier one gave; a failing entry leaves the project byte for byte unchanged; a dry run plans each entry on what the one before would write (`tests/rename/dry_run_batch.rs`: plan == real batch); what the command line refuses before any server starts |
| Doctor | none (temporary folders) | `tests/cli/doctor.rs` | overview, per-language report, `--json`, wrong variable, rename and move without a server |
| Kotlin (JVM) | `tests/fixtures/kotlin/jvm_project/` | `tests/kotlin/moves.rs`, `tests/kotlin/rename.rs`, `tests/kotlin/server.rs`, `tests/kotlin/dispatch.rs` | package moves, directory moves, rollback, symbol renames, clash and shadowing refusals, dispatch through the CLI entry points |
| Kotlin (Android) | `tests/fixtures/kotlin/android_project/` | `tests/kotlin/android.rs` | class moves and renames with manifest, layout and navigation XML, `R` and `BuildConfig` imports, compiled with a real Android Gradle Plugin |

### Batch tests

`tests/moves/batch.rs` exercises multi-file invocations using the real CLI binary. It reuses the same per-language fixture directories and covers:

| Scenario | What is tested |
|---|---|
| TypeScript: unrelated pair | Two files with no import relationship — placement only |
| TypeScript: cross-importing pair | Both files move to the same dir; import between them is rewritten |
| Python: two files with cross-import | Sequential Rope moves; second move sees the already-updated import |
| Rust: two same-dir files | Single rust-analyzer session; project still compiles after batch |
| Go: same-package batch | Both files from one package — gopls moves the whole package once |
| Go: cross-package batch | Files from two different packages — one gopls session for both packages |
| Dart: two files | Cross-import between them is rewritten |
| Markdown: two files | Links between them are rewritten |
| Mixed language (TS + Markdown) | Two languages dispatched independently in one CLI call |
| Partial failure | One language succeeds, one fails — response reports both |
| All fail | Exit non-zero with structured error message |

### Rename tests for Go, Rust, Python, and Dart

These scenarios start the real language server, so they are `#[ignore]`d and a plain `cargo test` skips them. A server that is not installed does not skip them: they fail with refac's own missing-server explanation (where it looked, the environment variable, `Run refac doctor <language>`), so the same message that teaches an agent also teaches the test runner. Each successful scenario ends by running the renamed project, because a rename is right when the project still behaves.

```bash
cargo test --test rename -- --ignored
```

The Dart suite also has one normal test (a project without `package_config.json`), so use `--include-ignored` there. Needs `gopls`, `rust-analyzer` (a toolchain that has it; the tests copy `rust-toolchain.toml`), `basedpyright`, and the Dart SDK on `PATH` or through `REFAC_GOPLS`, `REFAC_RUST_ANALYZER`, `REFAC_PYTHON_SERVER`, `REFAC_DART`. `tests/cli/doctor.rs` needs none of them: it runs the binary with an empty `PATH` and `HOME`. Under heavy CPU load gopls sometimes answers with part of the edits; the engine asks again (four attempts for Go), and the nine Go scenarios passed 16 of 16 runs under load. `REFAC_LSP_TRACE=1` prints every message exchanged with the server when a scenario misbehaves.

### Kotlin tests

**Do not call Kotlin tests like crazy.** They start a language server and a Gradle import (20 to 47 seconds, about 2 GB) and the whole group takes about 7 minutes, so a session that runs them after every edit spends its time waiting. Invoke them only when the change you made needs them. By default only the quick set runs, within 30 seconds (`cargo test --test kotlin quick:: -- --ignored --test-threads=1`, 25 to 28 s cold, 17 s warm); every other Kotlin test is **blocked** (it fails at once with the reason) unless the command says `REFAC_KOTLIN_TESTS=all`.

The Kotlin scenarios use the real JetBrains Kotlin language server and a Gradle import (20 to 47 seconds), so they are `#[ignore]`d and a plain `cargo test` skips them. **Put the tests you want into ONE command with `--test-threads=1`: one test, several named tests, or a module.** They share one server per fixture inside the command (`tests/common/pool.rs`): the first test of a fixture pays the start (20 to 32 seconds with the warm cache), every later one costs seconds, and a failed test costs no restart. The whole group (40 tests) takes about 7 minutes (419 s measured); only `server::` and the dry-run plans (the `refac` binary plans on a copy with a server of its own, about 25 seconds each) start servers of their own. The commands, the table of which tests cover which change, and the rules that keep the machine alive are in [Kotlin server setup](../Setup/kotlin_Server.md#running-the-real-server-tests):

```bash
export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0
export ANDROID_HOME=~/Android/Sdk   # Android tests only
# one test:
cargo test --test kotlin quick:: -- --ignored --test-threads=1
# one test outside the quick set (blocked unless you ask):
REFAC_KOTLIN_TESTS=all cargo test --test kotlin dispatch::a_kotlin_rename_is_routed_by_the_file_extension -- --ignored
# a few tests, one command, one after the other (tests of one fixture share its server):
REFAC_KOTLIN_TESTS=all cargo test --test kotlin -- --ignored --test-threads=1 moves::a_file_moves_to_a_new_package_and_every_reference_follows rename::a_class_is_renamed_together_with_its_file
```

Without `REFAC_KOTLIN_SERVER` (or `ANDROID_HOME` for the Android tests) these tests panic with an explanation instead of passing. Every successful scenario ends with a Gradle compile of the result, because a Kotlin refactor is right when the project still builds.

**Fixtures are never modified by running tests.** `common::setup_fixture` copies the fixture into a temp dir before each test. The tool operates on the temp copy; the originals stay pristine and the temp dir is cleaned up automatically when the test ends. No reset step is needed.

**Temp dir naming matters for Go.** `setup_fixture` uses the prefix `refac-test-` (visible, non-hidden directory). gopls skips workspace roots whose directory name starts with `.`, so hidden temp dirs (the `tempfile` crate's default `.tmp` prefix) prevent import cascade. Always use a visible prefix when testing Go moves.

**Dart tests are serialised.** The Dart analysis server is sensitive to concurrent starts. `dart_move.rs` acquires a global `Mutex` before each test so at most one analysis server runs at a time within that binary. `dart/package_config.rs` holds the single test for a project without `.dart_tool/package_config.json` (the move must be refused and every file left byte-identical); the refusal happens before any server starts, so it needs no lock.

**No test waits a fixed time for a language server.** The Dart, Go, Rust and Pyrefly drivers wait for the server's own readiness signal (see the shared client in `src/drivers/lsp/client.rs`), so the suites pass on a busy machine too: the Dart, Go, batch, Rust and Python suites were run under 40 busy processes on 4 cores and passed. A server that never becomes ready fails after `REFAC_LSP_TIMEOUT_SECS` (default 300); set it lower when debugging a hang.

**Pyrefly is a fallback behind Rope**, so `python_move` normally never starts it. `cargo test --lib pyrefly -- --ignored` runs the real server (it needs `.venv/bin/pyrefly` or `pyrefly` on `PATH` and panics when it is missing).

Run a single language's tests:

```bash
cargo test go_move
cargo test dart_move
```

**To add a new fixture file:** drop it in the fixture directory, then add assertions in the corresponding `*_move.rs`. Follow the pattern in the file: snapshot control files before the move, assert positive changes after, use comments to document limitations explicitly.

## 3. Manual CLI Verification

Generate sample projects:

```bash
cargo run --bin create_testbed
```

Then run the CLI against one concrete package root:

```bash
./target/debug/refac move \
  --project-path Trials/0_Refac_Tree/typescript \
  --source-path src/models/TaskManager.ts \
  --target-path src/core/TaskManager.ts
```

For machine-readable output:

```bash
./target/debug/refac move \
  --json \
  --project-path Trials/0_Refac_Tree/go \
  --source-path pkg/service/ledger.go \
  --target-path pkg/ledger/ledger.go
```

## 4. Project-Level Validation

After a move, validate the affected project with its native toolchain when possible:

- Rust: `cargo check`
- Go: `go build ./...`
- Python: import the affected modules or run project tests
- TypeScript: run the package typecheck/build if available
- Dart: run the package analyzer/build if available
- Kotlin: `./gradlew compileKotlin compileJava` (or the project's usual build); read the `// Note:` lines of the output for old names refac does not edit

## 5. Debugging Notes

### TypeScript scans are too broad

Check the project scope when TypeScript work becomes slow or memory-heavy. The Oxc backend scans all configured callers without a compiler Program or type checker. The helper stops on its time or sampled RSS limit; see [TypeScript process limits](../Features/TypeScript/linker_TypeScript.md#key-limits). A limit failure requires inspecting the working tree before retrying; one-file batches still scan the complete configured project.

- Good: point `--project-path` at the concrete TypeScript package that owns `tsconfig.json`
- Bad: point `--project-path` at a monorepo root and pass long nested paths

### Source path does not exist

`--source-path` is resolved relative to `--project-path`. If the file is real but the tool cannot find it, the root is usually wrong.

### Backend-specific tooling is missing

Each language backend depends on external tooling:

- TypeScript: `bun`
- Python: `rope` importable from `.venv/bin/python` (or system `python3`). The driver uses Rope by default and falls back to Pyrefly if Rope is unavailable.
- Rust: `rust-analyzer`
- Go: `gopls`
- Dart: `dart`
- Kotlin: the JetBrains Kotlin language server (`REFAC_KOTLIN_SERVER`) and a JDK 17+; Android projects also need `ANDROID_HOME`. See [Kotlin server setup](../Setup/kotlin_Server.md).
