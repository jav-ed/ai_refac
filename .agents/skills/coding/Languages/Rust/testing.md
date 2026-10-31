# Rust testing

What Cargo caches, what it does not, which runner to pick, and when a result cache such as Moon earns its startup. Read it after a test is justified (see [Testing](../../Tooling/Testing/testing.md)); to cache Rust tasks with Moon, go on to [Moon for Rust](moon_Cache.md).

Cargo caches compiling and linting in `target/`, but it never caches a test result. Rust is in the same position as `pytest`, unlike `go test` and Gradle. The numbers below come from one small service crate (11 tests, 0.3 s per run) on a 12-core machine with Cargo 1.98, cargo-nextest 0.9.143 and Moon 2.5.3. They show the size of an effect, not what your machine will do. Strength labels: **Contract** (a result is wrong without it), **Default** (follow unless you have a reason), **Starting point** (adapt freely).

## Default: `cargo test --locked`

`--locked` fails when `Cargo.lock` is out of date instead of silently re-resolving. The command runs unit tests, every file in `tests/` and the doc tests. Each test binary runs on its own and the tests inside it run as threads of one process. The checks that belong next to it:

| Check | Command | Cache result? |
| --- | --- | --- |
| Format | `cargo fmt --check` | Cheap (0.1 s), nothing to gain |
| Lints | `cargo clippy --locked --all-targets -- -D warnings` (tests too) | Cargo already skips unchanged work: 0.14 to 0.25 s unchanged, 22.6 s on a cold `target/` |
| Tests | `cargo test --locked` | Cargo never caches the result: the only gap a result cache can fill |
| Advisories, policy | `cargo audit --deny warnings`, `cargo deny check` | Never: they read an advisory database that changes daily, so the files do not decide the result |

Starting point for large crates: `debug = false` and `incremental = false` in `[profile.dev]`. One workspace measured disk 2.5 GB to 0.58 GB and a clean build 51 s to 35 s, at the price of a one-line rebuild going from 2 s to 7 s. Gap: not measured on the crate above.

## nextest: a Starting point, not the Default

`cargo nextest run --locked` runs every test in its own process: failures are isolated, timeouts work per test and the output is clearer. Two things to know before adopting it.

- **Contract: a test helper must not assume "once per test binary".** A `OnceLock` initialises once per process, which under nextest is once per test. A helper that writes a shared fixture file therefore writes it several times at once, and a reader that opens it between the truncate and the write sees an empty file. Measured: 6 of 25 full nextest runs failed this way and none of 25 `cargo test` runs. Writing to a file of the process's own name and renaming it over the target (atomic) fixed it: 60 of 60 nextest runs passed. Run a suite a few dozen times under nextest before relying on it, and give every helper that writes shared state an atomic write or a per-process path.
- **It does not run doc tests**, so keep `cargo test --doc` beside it. Gap: the crate above has none, so this is nextest's documented behaviour, not something exercised here.

Speed: on 11 tests in 0.3 s, nextest took 0.4 to 0.6 s against 0.26 to 0.32 s for `cargo test`, because a process starts per test. Gap: a large or slow suite was not measured; that is where per-test processes pay.

## Layout of a large crate

Every file directly in `tests/` is compiled and linked as its own test program. One large crate measured about 90 MB per program, which is why it groups integration tests into `tests/<group>/main.rs` with the group's files as modules. Starting point; Gap: not measured here, the crate above has two test files.

## Rate each test

Apply the [Testing](../../Tooling/Testing/testing.md) rules: justify it, rate its benefit from 1 to 10, and write the rule it protects and the rating in a comment above the test, so the next reader can judge whether it still earns its place. A Rust test that runs in milliseconds is uncached but light, so the lighter bar for an uncached, cheap test applies (5, if the repository records it). In the 11-test crate, nine stayed (contract file, response shape, byte-exact bodies, loopback-only bind, clean exit on SIGTERM, exit code of a validation command, each rated 5 to 9) and two went: the exit code clap already guarantees for a missing command (2) and generated completions and manual pages (4, a cosmetic loss that shows up on first use). Tests that pin your own contract or security boundary stay; tests of a library's own behaviour go.

## Moon or not

| Situation | Choice |
| --- | --- |
| One crate or service whose whole gate runs in a few seconds | No Moon. Measured: a replayed test task took 0.22 to 0.46 s against 0.28 s for running the tests, and a replayed fmt, clippy and test gate 0.66 to 0.97 s against about 0.7 s natively. Executing a task through Moon cost about 1 s. |
| Deterministic tests that take many seconds per run | Moon test task: the replay costs the same fraction of a second, whatever the run cost. |
| Several crates, or Rust feeding other languages (export an API document, generate a client, typecheck it) | Moon: it holds the ordering and re-runs exactly the dependent tasks. One earlier trial of such a chain replayed 35 of 36 tasks in 0.7 s. Gap: no native warm baseline was measured for it. |
| The repository already uses Moon | Add the Rust tasks the same way. |

Moon does not cache the expensive part of Rust, the compile: a cold clippy took 22 s with and without it. Never cache `target/` as a Moon output.

## Navigation

- [Moon for Rust](moon_Cache.md): the tested `moon.yml`, the input, dependency-edge, environment and failure rules that keep a cached Rust result true, the results of the six cache checks with cargo-call counts, timings, and what is not verified. Read it when adding Moon tasks to a Rust crate or workspace, or when a Rust result looks stale.
