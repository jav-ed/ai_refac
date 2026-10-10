# Kotlin Language Server Setup

Kotlin moves and renames need the JetBrains Kotlin language server. It is the one external dependency of the Kotlin backend, and refac does not download it for you: a missing or broken install is an error that carries the steps below. Everything else (Gradle, a JDK, the Android SDK for Android projects) is whatever the target project already needs to build.

## Install

The verified build is `ILS-263.6379.0` for Linux x64 (368,488,700 bytes, bundled JetBrains Runtime 25).

```bash
curl -LO https://download.jetbrains.com/language-server/kotlin-server/263.6379.0/kotlin-server-263.6379.0.tar.gz
echo "ab8ca4455dc2fc5fe1a24db2bccc46c104254d2c465155c4251ee65df8f3f7cc  kotlin-server-263.6379.0.tar.gz" | sha256sum -c -
mkdir -p ~/.local/share/refac
tar -xzf kotlin-server-263.6379.0.tar.gz -C ~/.local/share/refac
export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0
```

`REFAC_KOTLIN_SERVER` must name the folder that holds `bin/intellij-server` and `build.txt`, and `build.txt` must start with `ILS-`. Put the export in the shell profile of whoever runs `refac`, including an agent's environment. The folder can be anywhere: the install above uses `~/.local/share/refac`, and a machine that keeps its tools elsewhere (for example `/home/jav/Progs/kotlin-lsp/kotlin-server-263.6379.0`) only changes the path in the export. `refac doctor kotlin` prints `[ready]` and the found build when the variable is right.

A JDK 17 or newer must be on `PATH` for the Gradle import that the server runs. For Android projects `ANDROID_HOME` (or `local.properties`) must point at an SDK, exactly as for a normal Gradle build.

## Behavior to know

- **Startup cost:** every `refac` call on Kotlin starts the server, waits for the Gradle import to finish, then works. A small project is ready after about 30 seconds and each request adds seconds, so a Kotlin call is a batch operation. Put several moves into one `move` call instead of many calls. Measured on 2026-10-07 with the `tests/fixtures/kotlin/jvm_project`: a `rename --dry-run` took 37 to 38 seconds end to end, and the server process peaked at about 1.6 GiB resident memory (the project's own Gradle daemon comes on top). Refac sets no memory cap on this server, unlike the TypeScript engine; the JVM sizes its own heap. Real Android projects will need more, and the first measurement on one is worth taking.
- **Readiness is signalled, not guessed:** refac waits for the server's `intellij/workspaceImportState` to finish with every folder successful, then for `intellij/ready-for-test`. A broken Gradle build ends the wait early with the tail of the import log; otherwise the wait ends at `REFAC_KOTLIN_TIMEOUT_SECS` (default 600) with an error.
- **Scratch state:** the server needs a system path. Refac gives it a temporary directory (about 243 MB) and deletes it afterwards. A persistent system path was tried and gave no speedup, because the Gradle import dominates.
- **License:** this build asks for no EULA and needs no license. Its bundled EAP key is valid through 2026-10-30. When a later start fails with a license message, install a newer build, run the real-server tests below against it, and only then update the build constants in `src/drivers/kotlin/server.rs`.
- **Offline:** the Gradle import needs whatever the project needs to resolve its dependencies; refac adds no network use of its own. The one exception is a Kotlin Multiplatform build: the server cannot import it, so refac imports a plain-JVM copy of its sources instead (see [Kotlin Multiplatform](../Features/Kotlin/multiplatform_Mirror.md)), which needs the Kotlin Gradle plugin of the project's Kotlin version from the plugin portal or Maven Central, and the version from `gradle/libs.versions.toml`, the plugin line of a build script, or `REFAC_KOTLIN_MIRROR_VERSION`.

## Running the real-server tests

The tests that use the server are `#[ignore]`d so a plain `cargo test` stays fast and offline. Without `REFAC_KOTLIN_SERVER` they panic with this page's path instead of passing silently.

**The rule: put the Kotlin tests you want into ONE `cargo test` command with `--test-threads=1`: a single test, several named tests, or a module. All of them share one server per fixture, so the command pays the 40-second start once per fixture and every further test costs 1 to 5 seconds.**

**How the sharing works** (`tests/common/pool.rs`, `src/drivers/kotlin/server/lend.rs`):

- The first test of a fixture (`jvm_project`, `android_project`, `kmp_project`) starts the Kotlin server and imports the Gradle project (about 40 seconds: 4 JVM, 31 Gradle import, 6 indexing). The server is lent (`server::lend`) for the project directory, and the entry points of the library (`move_files`, `rename_symbol`, `rename_all_symbols`, `handle_refactor`, `handle_rename`) use a lent server instead of starting their own. The tests call those entry points exactly as the tool does; nothing in the tests is a copy of the engine.
- A test takes a lease on the pool, so tests that share servers run one after the other. When the lease starts, the pool puts the project directory back as the fixture was and `recover` tells the server what the disk has for every file it was ever shown: every open document is closed, a file that exists is announced as new, one that is gone as deleted (`resync::follow_disk`). Whatever the test before did, a failed operation included, the next test starts from a project that equals a fresh copy, with a server that agrees with it. No restart is ever needed.
- The servers of all fixtures that were used stay alive until the test program ends (about 2 GB each, so up to 6 GB with a Gradle daemon each), and an `atexit` hook stops them. The compile check that ends a successful scenario runs on a Gradle daemon the pool keeps (`--rerun-tasks`, 2 seconds instead of 17 with a new daemon), which stops with the server and by itself three minutes after its last use.

**What starts a server of its own:** `server::` (the start and the failing import themselves: 18 and 37 seconds) and the dry-run plans (`dry_run::`, `multiplatform::the_plan_...`, `rename::a_dry_run_of_a_batch_plans_...`). A plan is made by the real `refac` binary on a throw-away copy of a fresh fixture, which needs its own import of that copy (about 50 seconds each); the real move that the plan is compared with runs on the shared server.

Measured on 2026-10-10 (4 cores, 16 GB): the whole group of 38 tests takes 534 s (9 minutes) in one command (it took 2355 s when every test started its own server). After the start the 8 `moves::` tests take 25 s together, the 10 `rename::` tests that need no copy about 25 s, the 5 `android::` tests 45 s (they compile an Android build after each scenario).

```bash
export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0
export ANDROID_HOME=~/Android/Sdk   # only the Android tests need it
# (in a script: export them in the SAME shell that runs cargo, not inside a pipe)

# 1. ONE test (about 1 minute, nearly all of it the server start):
cargo test --test kotlin moves::a_file_moves_to_a_new_package_and_every_reference_follows -- --ignored

# 2. SEVERAL tests in ONE command: list the names after `--`. Tests of one fixture share its server:
cargo test --test kotlin -- --ignored --test-threads=1 \
  moves::a_file_moves_to_a_new_package_and_every_reference_follows \
  rename::a_class_is_renamed_together_with_its_file \
  dispatch::a_kotlin_rename_is_routed_by_the_file_extension

# 3. One MODULE (a name that ends in ::) when the change is about that area only:
cargo test --test kotlin moves:: -- --ignored --test-threads=1
```

`--nocapture` adds a `[pool]` line per test: `Kotlin server started in 37s` once, `reused` for the others. If you see more than one start per fixture, the sharing is broken.

Which tests for which change:

| The change touches | Run these (one command, form 2 or 3) |
|---|---|
| `server.rs`, `server/install.rs`, the start or readiness of the server | `server::` (2 tests) and `dispatch::a_kotlin_rename_is_routed_by_the_file_extension` |
| Kotlin moves (`moves.rs`, `plan.rs`, `declarations.rs`) | the module `moves::` (8 tests, about 65 seconds with the start) |
| Kotlin symbol rename (`rename.rs`) | the module `rename::` (shares the server; the batch dry run adds 50 s), or `rename::a_class_is_renamed_together_with_its_file` and `rename::a_clash_with_a_member_in_the_same_class_is_refused` |
| The Android layer (`android/`) | the module `android::` (5 tests) and `dry_run::the_plan_of_an_android_move_includes_the_manifest_and_layout_edits` |
| The dry run (`preview/copy*`, `plan_move`) | `dry_run::the_plan_of_a_package_move_names_every_file_whose_import_changes` |
| The Kotlin Multiplatform mirror (`server/mirror*`) | the module `multiplatform::` (7 tests; the refusals and the failure test need no server) |
| The shared server of the tests (`tests/common/pool.rs`, `server/lend.rs`, `resync.rs`) | `moves::` and `rename::` in one command, then `--nocapture` to count the starts |

A project without a `commonMain`, `commonTest` or `<target>Main` source set never gets a mirror, so a change that only touches the mirror cannot affect the plain JVM or Android tests. Run the whole group (`cargo test --test kotlin -- --ignored --test-threads=1`, nothing else running) once before a push that changes the server start or the shared move and rename engine, not repeatedly.

Rules that keep the machine alive:

- **Do not build while a Kotlin test runs.** The plan tests and `dispatch::` spawn `target/debug/refac`; a rebuild under them breaks them. Edit docs, not code, while one runs.
- **Never run two Kotlin test commands at the same time**, and keep `--test-threads=1`: the pool serialises the tests that use it, but the tests that start a server of their own do not wait for it, and two programs would hold 4 to 12 GB of servers at once.
- **The tool itself:** put all files of one change into one `refac move` call. That is one server and one Gradle import instead of one per file; a command still stops its server when it ends.
- A killed test program cannot stop its servers (the pool stops them when the program ends normally). After a run, `ps aux | grep -i -E 'gradle|intellij' | grep -v grep` must print nothing; the compile daemon of the pool stops itself three minutes after its last use.

See [Testing & Debugging](../Guides/Testing_and_Debugging.md) for the full test map and [Resource use](resource_Use.md) for the memory numbers.
