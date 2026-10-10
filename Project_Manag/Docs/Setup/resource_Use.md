# Resource use: disk and memory

This repository is small: about 3 MB of tracked files and an 8 MB `.git`. Everything big is a by-product that a tool creates and that can be deleted and made again: the Rust build output, language-server downloads, package caches. Nothing in this project needs 100 GB; a build from nothing writes 2.1 GB and peaks at 3.2 GB of RAM. This page explains where the space and the memory go, the rules that keep them small, and what to delete when a disk is full. It applies to **every tool** an agent starts here (Rust, Go, Gradle, Dart, Python, Bun), not only to Rust.

**Disk and memory are different limits.** A cloud sandbox has a fixed *disk allowance* per session (`df` can show free space and still be wrong, because the allowance is spent first) and a separate amount of RAM. "No space left on device", a linker "Bus error" or a failing `cargo test` build means the *disk* allowance is spent. Slow swapping or a killed process means RAM. They have different fixes.

## What is big, measured

| What | Size | Made by | Safe to delete? |
| --- | --- | --- | --- |
| tracked files | 3 MB | git | no |
| `.git` | 8 MB | git | no |
| `target/release` (the installed binary, 36 MB, plus 1.5 GB of build cache) | 1.5 GB | `cargo build --release`, `scripts/install.sh` | only to reinstall: it holds the binary the `~/.local/bin/refac` link points to; `scripts/install.sh` rebuilds it in about 4 minutes |
| `target/` after a fresh build | 2.1 GB | `cargo build`, `cargo test` | yes, rebuilds in about 2.5 minutes |
| `target/debug` after weeks of builds | 4.4 GB or more (6.3 GB measured on 2026-10-09) | dead cache: old dependency copies after a toolchain or Cargo.lock change, and one more copy of every test program after each source change | yes, `cargo clean -p refac` removes this crate's old copies, `rm -rf target/debug` removes everything; the next test build takes about 2.5 minutes |
| `~/.rustup` | 2.1 GB | the Rust toolchain | only to reinstall |
| `~/.gradle` | 2.0 GB | Kotlin and Android tests | keep: the Maven mirror of a sandbox can answer 429 when everything is downloaded again |
| `~/.cache/go-build` | 0.74 GB | gopls, Go tests | yes: `go clean -cache` |
| `~/go` | 0.8 GB | the Go toolchain and gopls | only to reinstall |
| Kotlin language server | 1.2 GB unpacked | `refac doctor kotlin` steps | only to reinstall |
| Android SDK for the Android tests | 0.5 GB | the tests' own setup | only to reinstall |

## Building refac

### Disk

The library embeds rust-analyzer (`ra_ap_*`), and **every file directly inside `tests/` is its own test program that links the whole library**. With cargo's default full debug information one such program is about 560 MB. The old layout had about 45 of them: a full `cargo test` build wrote **19 GB** and stopped with a linker "Bus error" when the sandbox's disk allowance ran out. Two changes bring it to 2.1 GB:

1. **Debug information is cut in `Cargo.toml`.** Dependencies get none (`debug = false`), this crate keeps line tables (`debug = "line-tables-only"`), so a panic and `RUST_BACKTRACE=1` still name file and line. A test program drops from 560 MB to about 90 MB. The profile is committed, so a plain `cargo build` and `cargo test` are lean without any environment variable.
2. **Test programs are grouped.** `tests/<group>/main.rs` is one program with the files of its group as modules: `rename`, `moves`, `kotlin`, `typescript`, `markdown`, `cli` (9 programs including the unit-test ones, about 100 MB each, instead of about 45). **Never add a `.rs` file directly in `tests/`**: it becomes another 90 MB program. Add a module to the group it belongs to (and `mod name;` in that group's `main.rs`), or start a new group folder when it is a new subject. `tests/cli/layout.rs` fails the build of the next change when a `.rs` file sits directly in `tests/`.

What the 2.1 GB is: dependencies about 1.8 GB (the `ra_ap_*` crates are the largest: `hir_ty` 126 MB, `hir_def` 60 MB), the nine test programs and the library the rest.

### Memory

A from-scratch `cargo test --no-run` measured on a 16 GB machine: **peak 3.2 GB**, 145 seconds. The peak is the linker (`rust-lld` 1.9 GB, while it links several test programs at once) plus `rustc` 1.2 GB. A rebuild after an edit is much smaller. If RAM is the limit, `cargo build -j 2` (or `CARGO_BUILD_JOBS=2`) links fewer programs at once, at the price of time.

### Rules for any Rust work here

- One `target/` directory. Experiments with a different profile or `RUSTFLAGS` write a **full second copy**; if you must, set `CARGO_TARGET_DIR` outside the repository and delete it afterwards.
- Do not pass `CARGO_PROFILE_*_DEBUG` for ordinary work. A different debug setting rebuilds everything (and doubles `target/`). For a debugger session use `CARGO_PROFILE_DEV_DEBUG=true cargo test --test <group>` and delete `target/` afterwards.
- Look at the size after a long session: `du -sh target`. Above 3 GB, delete what is stale: `cargo clean -p refac` (only this crate, dependencies stay), then `rm -rf target/debug/incremental`. When it is far above, `rm -rf target` and rebuild once (about 2.5 minutes).
- Run the tests you need (`cargo test --test rename go_`), not `cargo test --workspace --all-targets` every few minutes.
- `cargo clippy` and `cargo check` use their own artifacts in the same `target/`; they are small, but `cargo clippy --all-targets` compiles the test programs again as `check` builds.

## Running refac

refac starts a language server for the command and stops it when the command ends, so **nothing stays resident**. Memory is held only while a command runs, measured on small projects:

| Server | Start | Memory |
| --- | --- | --- |
| gopls | 1.7 s | 165 MB |
| rust-analyzer | 4.6 s | 637 MB, grows with the project |
| basedpyright | 1.3 s | 158 MB |
| Dart analysis server | 0.4 s | 124 MB |
| Kotlin server | about 38 s | 1.3 to 1.8 GB |
| TypeScript helper | seconds | limited to 4096 MiB by `REFAC_TYPESCRIPT_MAX_RSS_MB` (the command fails loudly above it) |

Two leaks were found and closed, and the pattern applies to every new tool:

- The Kotlin server imports the Gradle build through a **Gradle daemon**, which Gradle keeps for **three hours** (about 480 MB of RAM) after the command. refac starts the server with `JAVA_TOOL_OPTIONS=-Dorg.gradle.daemon.idletimeout=10000` so the daemon stops itself ten seconds after the import. The tests build with `./gradlew --no-daemon`.
- The Kotlin server's system folder (about 250 MB of caches) is a throwaway temporary folder deleted at the end.

## Tests with real servers

They are `#[ignore]`d, so a plain `cargo test` does not start any server. Start them in groups (`Guides/Testing_and_Debugging.md` has the commands). Each one runs one server at a time with `--test-threads=1`. The Kotlin group is the exception: its tests share one server per fixture inside one command, and the servers of all fixtures that were used stay alive until the test program ends (up to three, 1.5 to 2 GB each, plus a Gradle daemon for the compile checks), while `server::` and the dry-run plans start one more of their own for a minute. Put the Kotlin tests you want into one command ([how](kotlin_Server.md#running-the-real-server-tests)) and never two commands at once. After a Kotlin run check that nothing survived: `ps aux | grep -i -E 'gradle|intellij|gopls|rust-analyzer|pyright|dart' | grep -v grep` must print nothing.

## When the disk is full

Symptoms: "No space left on device", "Bus error" in the linker, a test program that is cut off. Delete, in this order, until builds work, and say what you deleted:

1. `rm -rf target/debug/incremental` and stale test programs (`find target/debug/deps -maxdepth 1 -type f -size +50M`: anything that is not one of the current group names).
2. `cargo clean -p refac`.
3. `go clean -cache`.
4. `rm -rf target` (rebuilds in about 2.5 minutes).
5. Throwaway experiment folders you created (scratchpad copies of fixtures, `fresh_target`-style build directories).

## Adding a tool: the checklist

1. **Start it for the command and stop it at the end**; never leave a process behind. If the tool starts a daemon (Gradle, a build server, a language server of its own), set its idle timeout to seconds or pass its `--no-daemon` flag.
2. **Measure** the memory (sum the resident size of the whole process tree once a second; a child process is the usual surprise) and the disk it writes, on a small project and a bigger one, and write the numbers here and in `Setup/language_Servers.md`.
3. **Cap it** where the tool supports a cap, and make exceeding it a loud error (the TypeScript helper's `REFAC_TYPESCRIPT_MAX_RSS_MB` is the example).
4. **Do not keep caches** that give no speedup (the Kotlin system folder was measured and gave none, so it is deleted).
5. **Never add debug information or optimisation levels** to a profile without measuring the size of the test programs first.

Related: [Language servers](language_Servers.md): where each server is found and the measured cost of a rename. [Kotlin server setup](kotlin_Server.md): the large download. [Testing guide](../Guides/Testing_and_Debugging.md): how to run the suites.
