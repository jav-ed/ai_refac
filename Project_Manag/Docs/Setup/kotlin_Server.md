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

- **Startup cost:** every `refac` call on Kotlin starts the server, waits for the Gradle import and the indexing to finish, then works, so a Kotlin call is a batch operation. Put several moves into one `move` call instead of many calls: a single Kotlin move or rename is refused by default, with the batch command for the request and a long explanation of why Kotlin is slow and which options exist (`src/logic/kotlin_cost/explanation.txt`, the same text as `refac guide kotlin`). Where the time goes, measured on 2026-10-10 with the `tests/fixtures/kotlin/jvm_project` (server ready after 32 to 38 s on an empty cache): JVM and `initialize` 4 s, Gradle import 3 s with a running Gradle daemon and 11 s with a cold one, then **about 19 s of indexing the JDK and the Kotlin libraries**, then 6 s until the server signals ready. The index is the big part, not the Gradle import, and it is the same for every project, so the next item removes it. End to end a `refac move` of one file takes about 44 s on the first call and about 24 s on the calls after it (measured: 25.0 s, then 23.6 s after the Gradle daemon had stopped). The server process peaks at about 1.6 GiB resident memory (the project's own Gradle daemon comes on top). Refac sets no memory cap on this server, unlike the TypeScript engine; the JVM sizes its own heap. Real Android projects will need more, and the first measurement on one is worth taking.
- **Gradle daemon, off by default:** the server's Gradle import starts a Gradle daemon, which Gradle keeps for three hours; refac sets its idle time to 10 s so nothing stays after the command. `REFAC_KOTLIN_GRADLE_IDLE_SECS=<1 to 3600>` keeps it that many seconds instead, and the next command skips the cold import: measured with two consecutive `refac move` calls, 23.5 s then **15.1 s** with 120, against 25.0 s and 23.6 s by default. It costs the daemon's memory (several hundred MB) for those seconds after every command, so it is the user's opt-in for scripts that must run single changes one after the other, and it is not on by default. A value that is not a whole number of seconds from 1 to 3600 is an error naming the variable (`install::parse_gradle_idle`).
- **Readiness is signalled, not guessed:** refac waits for the server's `intellij/workspaceImportState` to finish with every folder successful, then for `intellij/ready-for-test`. A broken Gradle build ends the wait early with the tail of the import log; otherwise the wait ends at `REFAC_KOTLIN_TIMEOUT_SECS` (default 600) with an error.
- **Warm cache:** the server writes its index (the JDK, the Kotlin libraries, what it learned of the project; 110 to 160 MB) into a system directory. Refac keeps it between calls (`src/drivers/kotlin/server/cache.rs`): a run starts from a private copy of the warm directory, so two commands never share a live one, and a run that ends cleanly (the server left by itself after `exit` and the end of its input) publishes its directory as the new warm one. A run that failed publishes nothing. The index is passed with `initializationOptions.indexDir`; without it the server keeps one full index per project path, and a copy of the project (a dry run, a test) would start empty. Measured: 32 to 38 s to ready with an empty directory, **9 s with a warm one and a running Gradle daemon, 16 s with a cold one**, for the same path and for another one. Edits, new files, deletions and moves made while no server ran are seen (checked with the references of a class after each). Where: `REFAC_KOTLIN_CACHE` (a directory, or `off` for an empty directory every time, as before), else `$XDG_CACHE_HOME/refac`, else `~/.cache/refac`, in a folder per server build. It is deleted when it grows past 2 GB. Delete it by hand (`rm -rf ~/.cache/refac/kotlin-*`) if a start misbehaves; a copy that cannot be read is an error that says so.
- **License:** this build asks for no EULA and needs no license. Its bundled EAP key is valid through 2026-10-30. When a later start fails with a license message, install a newer build, run the real-server tests below against it, and only then update the build constants in `src/drivers/kotlin/server.rs`.
- **Offline:** the Gradle import needs whatever the project needs to resolve its dependencies; refac adds no network use of its own. The one exception is a Kotlin Multiplatform build: the server cannot import it, so refac imports a plain-JVM copy of its sources instead (see [Kotlin Multiplatform](../Features/Kotlin/multiplatform_Mirror.md)), which needs the Kotlin Gradle plugin of the project's Kotlin version from the plugin portal or Maven Central, and the version from `gradle/libs.versions.toml`, the plugin line of a build script, or `REFAC_KOTLIN_MIRROR_VERSION`.

## Running the real-server tests

**Agents: do not call Kotlin tests like crazy.** A Kotlin test starts a language server (20 to 32 seconds before its first request, about 2 GB), and the whole group takes 7 minutes. Run them after every edit and your session is spent waiting. Invoke them only when the change you made actually needs them, once, with the few tests that cover it. If a Kotlin test cannot be made to fit into 30 seconds, it is not run by default: it is blocked and has to be asked for.

The tests that use the server are `#[ignore]`d so a plain `cargo test` stays fast and offline. Without `REFAC_KOTLIN_SERVER` they panic with this page's path instead of passing silently.

**Two tiers** (`tests/common/kotlin.rs`, `tests/kotlin/quick.rs`):

1. **The quick set, on by default:** `cargo test --test kotlin quick:: -- --ignored --test-threads=1`. Two tests on `jvm_project` (a file move whose Kotlin and Java importers follow, a member rename with a Java caller), one server start, no compile check. Measured on 2026-10-10: **27 s with a cold Gradle daemon, 17 s with a warm one** (start 20 to 23 s, or 10 s when the daemon of an earlier run is still up; each test then costs about 2 s). It must stay under 30 s: a third test (the rollback test) took the cold run to 29 to 33 s and was moved back to `moves::`. Time the whole set again before adding a test to it.
2. **Everything else is BLOCKED by default.** Every other Kotlin test calls `require_slow_tests()` first and, unless the command says `REFAC_KOTLIN_TESTS=all`, panics at once (0.0 s, no server started) with the quick command and the opt-in command. A value other than `all` is an error. Set it only for the tests your change needs:

```bash
REFAC_KOTLIN_TESTS=all cargo test --test kotlin -- --ignored --test-threads=1 <module::test> <module::test>
```

**The rule for the second tier: put the Kotlin tests you want into ONE `cargo test` command with `--test-threads=1`: a single test, several named tests, or a module. All of them share one server per fixture, so the command pays the start once per fixture (20 to 32 seconds with the warm cache, 40 to 47 without it) and every further test costs 1 to 5 seconds.**

**How the sharing works** (`tests/common/pool.rs`, `src/drivers/kotlin/server/lend.rs`):

- The first test of a fixture (`jvm_project`, `android_project`, `kmp_project`) starts the Kotlin server and imports the Gradle project (20 s for `jvm_project`, 31 to 32 s for `android_project` and `kmp_project` with the warm cache of the server, see "Warm cache" above; 40 to 47 s with an empty one). The warm directory is only updated when a server exits, so a pool server started in the first run on a machine is cold and the ones after it are not. The server is lent (`server::lend`) for the project directory, and the entry points of the library (`move_files`, `rename_symbol`, `rename_all_symbols`, `handle_refactor`, `handle_rename`) use a lent server instead of starting their own. The tests call those entry points exactly as the tool does; nothing in the tests is a copy of the engine.
- A test takes a lease on the pool, so tests that share servers run one after the other. When the lease starts, the pool puts the project directory back as the fixture was and `recover` tells the server what the disk has for every file it was ever shown: every open document is closed, a file that exists is announced as new, one that is gone as deleted, and the call waits until the server has dropped the deleted ones (`resync::follow_disk`; the server handles the deletion event about 0.2 s later, and a move into that folder in between is refused with "File Helper.kt already exists", which showed up as a failure in 2 of 5 runs once the server became fast). Whatever the test before did, a failed operation included, the next test starts from a project that equals a fresh copy, with a server that agrees with it. No restart is ever needed.
- The servers of all fixtures that were used stay alive until the test program ends (about 2 GB each, so up to 6 GB with a Gradle daemon each), and an `atexit` hook stops them. The compile check that ends a successful scenario runs on a Gradle daemon the pool keeps (`--rerun-tasks`, 2 seconds instead of 17 with a new daemon).
- **The Gradle daemon lingers for 120 s in the tests** (`GRADLE_IDLE_MS` in `pool.rs`, passed through `KotlinServer::start_with`). That is what makes a second run within two minutes start in 10 s instead of 20 s, and it means `ps` shows a Gradle daemon for up to two minutes after a test program ended. The tool itself does not do this: its daemon stops 10 s after the import unless the user sets `REFAC_KOTLIN_GRADLE_IDLE_SECS` (see the next list).

**What starts a server of its own:** `server::` (the start and the failing import themselves: 6 and 7 seconds with the warm cache) and the dry-run plans (`dry_run::`, `multiplatform::the_plan_...`, `rename::a_dry_run_of_a_batch_plans_...`). A plan is made by the real `refac` binary on a throw-away copy of a fresh fixture, which needs its own import of that copy (about 25 seconds each with the warm cache); the real move that the plan is compared with runs on the shared server.

Measured on 2026-10-10 (4 cores, 16 GB): the whole group of 38 tests takes **417 s (7 minutes) in one command** with a warm cache, 534 s with an empty one (it took 2355 s when every test started its own server and nothing was kept). By module, with the starts inside: `multiplatform::` 96 s, `dry_run::` 77 s, `rename::` 69 s, `android::` 68 s, `dispatch::` 36 s, `moves::` 26 s, `server::` 13 s. After the start the 8 `moves::` tests take 25 s together, the 10 `rename::` tests that need no copy about 25 s, the 5 `android::` tests 45 s (they compile an Android build after each scenario). Run the whole group once before a push that changes the server start or the shared move and rename engine, never repeatedly.

```bash
export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0
export ANDROID_HOME=~/Android/Sdk   # only the Android tests need it
# (in a script: export them in the SAME shell that runs cargo, not inside a pipe)

# 0. The quick set (30 s), the default:
cargo test --test kotlin quick:: -- --ignored --test-threads=1

# 1. ONE test outside it (about 1 minute, nearly all of it the server start):
REFAC_KOTLIN_TESTS=all cargo test --test kotlin moves::a_file_moves_to_a_new_package_and_every_reference_follows -- --ignored

# 2. SEVERAL tests in ONE command: list the names after `--`. Tests of one fixture share its server:
REFAC_KOTLIN_TESTS=all cargo test --test kotlin -- --ignored --test-threads=1 \
  moves::a_file_moves_to_a_new_package_and_every_reference_follows \
  rename::a_class_is_renamed_together_with_its_file \
  dispatch::a_kotlin_rename_is_routed_by_the_file_extension

# 3. One MODULE (a name that ends in ::) when the change is about that area only:
REFAC_KOTLIN_TESTS=all cargo test --test kotlin moves:: -- --ignored --test-threads=1
```

`--nocapture` adds a `[pool]` line per test: `Kotlin server started in 37s` once, `reused` for the others. If you see more than one start per fixture, the sharing is broken.

Which tests for which change (all of these need `REFAC_KOTLIN_TESTS=all`; the quick set needs nothing):

| The change touches | Run these (one command, form 2 or 3) |
|---|---|
| `server.rs`, `server/install.rs`, the start or readiness of the server | the quick set, then `server::` (2 tests) and `dispatch::a_kotlin_rename_is_routed_by_the_file_extension` |
| Kotlin moves (`moves.rs`, `plan.rs`, `declarations.rs`) | the quick set; the module `moves::` only when it changes the engine (8 tests, about 65 seconds with the start) |
| Kotlin symbol rename (`rename.rs`) | the quick set; `rename::a_class_is_renamed_together_with_its_file` and `rename::a_clash_with_a_member_in_the_same_class_is_refused`, the whole module `rename::` only for a large change (the batch dry run adds 25 s) |
| The Android layer (`android/`) | the module `android::` (5 tests) and `dry_run::the_plan_of_an_android_move_includes_the_manifest_and_layout_edits` |
| The dry run (`preview/copy*`, `plan_move`) | `dry_run::the_plan_of_a_package_move_names_every_file_whose_import_changes` |
| The Kotlin Multiplatform mirror (`server/mirror*`) | the module `multiplatform::` (7 tests; the refusals and the failure test need no server) |
| The shared server of the tests (`tests/common/pool.rs`, `server/lend.rs`, `resync.rs`) | `moves::` and `rename::` in one command, then `--nocapture` to count the starts |
| The refusal and its explanation (`src/logic/kotlin_cost*`, the guide topic) | no Kotlin test: `cargo test --test cli kotlin_cost` and `cargo test --lib kotlin_cost` need no server |

A project without a `commonMain`, `commonTest` or `<target>Main` source set never gets a mirror, so a change that only touches the mirror cannot affect the plain JVM or Android tests.

Rules that keep the machine alive:

- **Do not build while a Kotlin test runs.** The plan tests and `dispatch::` spawn `target/debug/refac`; a rebuild under them breaks them. Edit docs, not code, while one runs.
- **Never run two Kotlin test commands at the same time**, and keep `--test-threads=1`: the pool serialises the tests that use it, but the tests that start a server of their own do not wait for it, and two programs would hold 4 to 12 GB of servers at once.
- **The tool itself:** put all files of one change into one `refac move` call. That is one server and one Gradle import instead of one per file; a command still stops its server when it ends.
- A killed test program cannot stop its servers (the pool stops them when the program ends normally). After a run, `ps aux | grep -i -E 'gradle|intellij' | grep -v grep` must print nothing, except a Gradle daemon that stops by itself within two minutes of the end of the test program (the 120 s linger of the tests).

See [Testing & Debugging](../Guides/Testing_and_Debugging.md) for the full test map and [Resource use](resource_Use.md) for the memory numbers.
