# Moon for Rust

How to cache Rust checks with Moon so a replayed result can be trusted, and how to prove it. Read it after [Rust testing](testing.md) says a result cache pays, when adding Moon tasks to a Rust crate or workspace, or when a cached Rust result looks stale. The generic Moon rules are in [Moon](../../Tooling/Testing/Moon/moon.md) and [Make a task cacheable](../../Tooling/Testing/Moon/task_Setup.md); this page holds only what is specific to Rust.

Tested with Moon 2.5.3, Cargo 1.98, cargo-nextest 0.9.143, on a one-crate service (11 tests) and a throwaway two-crate workspace. Moon's Rust toolchain plugin has tier-2 support: it hashes `Cargo.toml` and `Cargo.lock` and knows `cargo`. Moon is not a build system here: Cargo owns compiling and `target/`, and rustup with `rust-toolchain.toml` own the compiler version.

## The setup (tested)

```yaml
# .moon/workspace.yml  (a workspace: projects: ['crates/*'])
projects:
  my-service: '.'
```

```yaml
# .moon/toolchains.yml: enables the Rust plugin; the compiler stays rustup's
rust: {}
```

```yaml
# moon.yml
language: 'rust'

fileGroups:
  sources:           # everything Cargo compiles or a test reads
    - 'src/**/*'
    - 'tests/**/*'
    - 'Cargo.toml'
    - 'Cargo.lock'
    - 'openapi.json' # a checked-in file the contract test compares against

tasks:
  fmt:
    command: 'cargo fmt --check'
    inputs: ['@group(sources)']
  clippy:
    command: 'cargo clippy --locked --all-targets -- -D warnings'
    inputs: ['@group(sources)']
  test:
    command: 'cargo test --locked'   # or: cargo nextest run --locked
    # A variable the tests read: a pass computed without it must not replay for a run with it.
    inputs: ['@group(sources)', '$MY_SERVICE_BLESS_OPENAPI']
  export-openapi:                    # a task that writes a file: declare it as an output
    script: 'mkdir -p generated && cargo run --locked --quiet -- openapi > generated/openapi.json'
    inputs: ['@group(sources)']
    outputs: ['generated/openapi.json']
  bless-openapi:                     # rewrites a checked-in file: never replay it
    command: 'cargo test --locked --test contract checked_in_openapi_matches_the_code'
    env: { MY_SERVICE_BLESS_OPENAPI: '1' }
    options: { cache: false }
  check:                             # names the gate; has no command of its own
    deps: ['~:fmt', '~:clippy', '~:test']
    options:
      cache: false                   # always evaluate the graph; the leaves stay cacheable
      runDepsInParallel: false       # stop at the first failure, no waiting on Cargo's build lock
```

Moon needs a git repository with at least one commit (it asks git for `HEAD`), and `.moon/cache/` and `target/` stay out of version control. Moon adds its own configuration files to every task's inputs, so editing `moon.yml` re-runs the tasks. Run `moon run my-service:check`; `moon task my-service:test` shows what a task hashes.

## Rules that keep a cached result true

1. **Contract: list every file the result depends on.** Sources, `tests/**`, `benches/**`, `examples/**`, `build.rs`, the manifest and lockfile (the workspace ones too) and `rust-toolchain.toml`, plus any file a test reads (a checked-in contract, fixtures). One earlier trial first left out `tests/**`, so a new test case replayed the old pass.
2. **Contract: a crate that compiles another crate's code must hash the edge.** Moon infers a Cargo path dependency written in the crate's own manifest, even when the package name differs from the project id. It infers nothing for a dependency inherited with `workspace = true`: declare `dependsOn: ['base']` in that case (verified: none inferred, then `explicit`). Inference alone is not enough: with the default `deps: ['^:test']`, a change to `base` re-ran only `base`'s tests, and `app`'s tests replayed a stale pass although they now fail (verified: `moon run` exited 0 against exit 101 on a real run). Use the hashed edge:

   ```yaml
   deps:
     - target: '^:test'
       cacheStrategy: 'hash'
   ```

3. **Environment:** a variable listed as `$NAME` in `inputs` changes the hash (verified: a run with it set re-ran, unsetting it replayed the earlier pass). An unlisted variable changes nothing, so list every one the tests read.
4. **Outputs:** a declared output is restored without running the command (verified: deleted, restored byte for byte, no cargo call). A test task has no output. Never declare `target/`.
5. **Never cache** `cargo audit`, `cargo deny` (an advisory database that changes daily), a task that rewrites checked-in files, or a test that needs a service, the clock or the network.
6. **Failures:** a failed run is never cached, and the run after a failure always executes, even when the inputs return to bytes that passed before (verified: the old pass was still in the cache, yet the run executed, and only the run after that replayed). That is the safe direction, so budget one extra run. With the default parallel dependencies a failing `fmt` does not stop `clippy` and `test`, which had already started; with `runDepsInParallel: false` only `fmt` ran (verified).
7. **Workspaces:** `cargo fmt` started in one member formats the whole workspace, so a half-written sibling crate can fail an unrelated crate's check; scope it with `--package <name>`. Reported by an earlier trial; Gap: not re-verified here.

## Prove it with an invocation log

Replayed console output proves nothing, because Moon prints the stored output of a cached run. Put a `cargo` shim first on `PATH` that appends one line per real call, then count lines after each step:

```bash
# shim/cargo
echo "$(date +%s.%N) cargo $*" >> "${INVOKE_LOG:?}"; exec "$HOME/.cargo/bin/cargo" "$@"
```

Results on the service crate (a cargo call count of 0 means the result was replayed):

| Step | Cargo calls |
| --- | --- |
| Identical repeat, three times | 0, 0, 0 |
| `moon run --force` | 1 |
| Unrelated files changed (`deny.toml`, `README.md`) | 0 |
| A listed input changed (`src/lib.rs`) | 1 |
| That input restored to its original bytes | 0 (replays the earlier pass) |
| A checked-in file the test reads edited: the test fails, as it should | 1, exit 101 |
| The same failing input again | 1 (failures are not cached) |
| Declared variable set / unset again / an unlisted variable set | 1 / 0 / 0 |
| Declared output deleted, task rerun | 0, file restored identical |
| Task with `cache: false`, twice | 1, 1 |

## Timings

| Run | Wall time |
| --- | --- |
| `cargo test`, warm | 0.26 to 0.32 s |
| `cargo nextest run`, warm | 0.4 to 0.6 s |
| `moon run` test task, replayed | 0.22 to 0.46 s |
| fmt, clippy and test replayed through `check` | 0.66 to 0.97 s |
| `moon run --force` test task (executes) | about 1.0 to 1.2 s |
| Cold `cargo test --no-run`, native | 30.6 s |
| Cold clippy, native / first run through Moon | 22.6 s / 22 s: Moon does not cache the compile |
| clippy again, nothing changed, native | 0.14 to 0.25 s |

Moon prints its own time ("Time: 32ms") on a replay. That excludes starting Moon, which takes 0.23 s without the Rust plugin and 0.33 s with it (five runs each). Judge a cache by wall-clock time, and quote it that way. Replay cost does not grow with the task, so the cache earns its place on slow deterministic tasks and on graphs, not on a suite that runs in 0.3 s.

## Not verified

Gap: installing `cargo-nextest` through the plugin's `bins` setting, pinning a compiler with `version:` in the toolchain configuration, a remote cache, a compile cache such as sccache, `--affected` and `moon ci` on Rust, and any machine other than the one above. Do not assume them; read the current Moon documentation for the installed version (`moon toolchain info rust` lists the Rust settings).
