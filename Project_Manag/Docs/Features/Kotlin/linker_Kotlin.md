# Kotlin and Android

Kotlin (JVM and Android) has two operations, both driven by the JetBrains Kotlin language server: **file and directory moves** that rewrite `package` lines, imports, and every same-package usage, and **symbol rename** (`refac rename`) for variables, functions, properties, classes, and members, including their Java callers. The server does the semantic work. Refac wraps it with planning, verification, rollback, and an Android layer for the parts the server never touches (manifest, layout and navigation XML, the implicit `R` and `BuildConfig`). Why this engine was chosen is in [Kotlin options](../../Investigation/kotlin_Options.md).

## What you need

A Gradle project (`settings.gradle(.kts)` in `--project-path`, which is the Gradle root and not a module folder), a JDK 17 or newer, and the Kotlin language server pointed to by `REFAC_KOTLIN_SERVER`. Android projects also need `ANDROID_HOME`. Install steps, environment variables, startup cost, and the license note are in [Kotlin server setup](../../Setup/kotlin_Server.md). Refac never downloads the server and never guesses: without it, every Kotlin call fails with the install steps.

## Pages

- [File moves](file_Moves.md): what a move updates, how requests are grouped for the server, the refusals (Java sources, cross-module moves, existing targets), the checks after the server answers, the all-or-nothing rollback, and the notes the command prints.
- [Symbol rename](symbol_Rename.md): `refac rename` for Kotlin: the command, how the symbol is chosen, the in-memory proof that a rename is faithful (name clashes and shadowing are refused), classes that are renamed together with their file, and the hard failures.
- [Android layer](android_Layer.md): what refac adds for Android after the server edit: class names in the manifest, layouts, and navigation graphs, `import <namespace>.R` and `BuildConfig` for files that leave their namespace package, and the report of old names it cannot rewrite (ProGuard, build scripts, string literals).

## Implementation owners

All Kotlin code is under [`src/drivers/kotlin/`](../../../../src/drivers/kotlin/), one file per job:

- `server.rs`: finds the install, starts the server, waits for readiness, owns the request timeout. Speaks LSP through the generic [`lsp_session.rs`](../../../../src/drivers/lsp_session.rs) that the TypeScript rename also uses.
- `plan.rs`, `moves.rs`, `moved.rs`, `checks.rs`, `declarations.rs`: move planning and refusals, the move run, the record of what moved where, and the cheap checks of the server's work.
- `rename.rs`, `rename/imports.rs`: the Kotlin `Language` for the shared rename engine in [`src/drivers/lsp_rename/`](../../../../src/drivers/lsp_rename/) (symbol discovery, the in-memory proof, `WorkspaceEdit` parsing, and the write with an undo log are the engine's, see [Engine](../Symbol_Rename/engine.md)). `rename.rs` says what is Kotlin about it: the server it starts, the import line the server drops, a class renamed with its file, the Android follow-up. `symbol_scan.rs` in [`src/drivers/`](../../../../src/drivers/) is shared with TypeScript.
- `android.rs` and `android/{namespace,xml,imports}.rs`, `survey.rs`, `renames.rs`, `stale.rs`: the Android layer and the report of names left behind.
- Dispatch: [`src/logic/route.rs`](../../../../src/logic/route.rs) sends `.kt` files and Kotlin directories here, and [`src/logic/rename.rs`](../../../../src/logic/rename.rs) sends `.kt` rename requests here.

## Tests

Pure logic (planning, XML rewriting, imports, edit parsing, verification helpers) runs in the normal `cargo test`. The scenarios against the real server are `#[ignore]`d and fail loudly when `REFAC_KOTLIN_SERVER` is missing: `tests/kotlin/server.rs` (startup, broken build), `tests/kotlin/moves.rs` (JVM moves, rollback), `tests/kotlin/rename.rs` (renames, clash and shadowing refusals), `tests/kotlin/android.rs` (a real Android Gradle Plugin project compiled after each move), and `tests/kotlin/dispatch.rs` (the CLI entry points). Fixtures are `tests/fixtures/kotlin/jvm_project` and `android_project`; every successful scenario ends with a Gradle compile, because a refactor is right when the project still builds.

## Known limits

- **Not supported:** moves of `.java` files (the server leaves their package line stale), directories that contain Java sources, moves between modules or source sets, and Kotlin Multiplatform projects.
- **Cost:** every call starts the server and imports the Gradle build, about 30 seconds and about 1.6 GiB for the server on a tiny project. Refac sets a time limit (`REFAC_KOTLIN_TIMEOUT_SECS`, default 600) but no memory cap, unlike the TypeScript engine.
- **Android build files are not edited:** ProGuard rules, `mainClass`, service lists, and string literals that name a moved class are reported, not rewritten.
- **Untested here:** cross-module Android projects with several libraries, and builds other than Gradle (Maven imports are the server's feature, but refac's Android layer reads Gradle scripts).
