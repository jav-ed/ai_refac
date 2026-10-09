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
- **Offline:** the Gradle import needs whatever the project needs to resolve its dependencies; refac adds no network use of its own.

## Running the real-server tests

The tests that use the server are `#[ignore]`d so a plain `cargo test` stays fast and offline. Run them with the variable set:

```bash
export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0
export ANDROID_HOME=~/Android/Sdk   # only the Android tests need it
# ONE test is enough to prove the server works with refac (about 2 minutes):
cargo test --test kotlin dispatch::a_kotlin_rename_is_routed_by_the_file_extension -- --ignored
```

Run a single test, and only when a change touches the Kotlin path or the server build changes. Every test starts the server and imports a Gradle project (about 40 seconds each, longer on the first run while Gradle fetches dependencies); on 2026-10-09 the three `dispatch::` tests took 385 seconds in total, which proved nothing the first one had not. Do not run the whole group (`--test-threads=2` over every Kotlin test) unless a Kotlin backend change needs it. Without the variable these tests panic with this page's path instead of passing silently. See [Testing & Debugging](../Guides/Testing_and_Debugging.md) for the full test map.
