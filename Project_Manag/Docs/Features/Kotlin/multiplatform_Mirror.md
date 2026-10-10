# Kotlin Multiplatform: the plain-JVM mirror

The Kotlin language server cannot read a Kotlin Multiplatform Gradle build. Its import of such a build ends "successfully" but prints `Failed to find 'target' in Kotlin extension`, no source set is known afterwards, and every move is refused with `Destination directory was not found (LSP error -32602)`. Its own documentation lists Multiplatform as "not yet" (see [Kotlin options](../../Investigation/kotlin_Options.md)). It was reproduced here on the smallest possible build: `kotlin("multiplatform")` with one `jvm()` target and sources under `src/commonMain/kotlin`.

The server does read a plain Kotlin/JVM build. So refac gives it one. When a Gradle root has a multiplatform source set, `KotlinServer::start` builds a **mirror** and starts the server on that instead, and nothing else in refac knows the difference: moves, renames, the Android layer, the rollback and the dry run all keep speaking in the real project's paths.

## When it applies

A source set named `commonMain`, `commonTest` or `<target>Main` (`jvmMain`, `androidMain`, `iosMain`, `desktopMain`) in any module below the Gradle root. Android and plain JVM modules use `main`, `test`, `androidTest` and the names of variants and flavors, so they never get a mirror and take the path that was tested before. A multiplatform build whose source sets do not follow those names is not recognised; the server's import then prints the message above and refac stops with an error that says so, rather than letting the server refuse every move.

## What the mirror is

A temporary directory with the same relative layout as the project, holding

- a `settings.gradle.kts` and a `build.gradle.kts` for one plain `kotlin("jvm")` module whose source folders are every `src/<source set>/kotlin` and `.../java` folder of every module of the real project (`commonMain`, `jvmMain`, `androidMain`, `commonTest`, an Android app module, all together),
- a copy of those `.kt` and `.java` files, and nothing else (the server reads nothing else),
- the Gradle wrapper files of the project, so the import uses the project's own Gradle version.

It is deleted when the command ends, like the server's own scratch directory.

## What is translated

`src/drivers/kotlin/server/mirror.rs` sits at the `KotlinServer` boundary, so every caller gets it:

- Every `file:` URI in a request goes to the server in the mirror's namespace, and every URI in the answer (including the URI keys of `WorkspaceEdit.changes`) comes back in the project's.
- `workspace/didChangeWatchedFiles` and the document texts refac sends after it keep the mirror like the project: moved files are removed and copied, edited files copied, events about files the mirror does not hold (resources, manifests) are not forwarded. A created file that is not on disk is an error.
- `expect` and `actual` modifiers are replaced by spaces of the same length in the mirror's copy of a file (`mirror/modifiers.rs`). In a plain JVM module the server takes an `expect` declaration for one "moved to a platform module" and refuses it. Because the text keeps its length, every line and column of the mirror is the line and column of the real file. The server's edits are applied to the real files as they are, and an edit that would replace text with such a modifier is refused (the mirror hides the modifier from the server, so refac would overwrite it).

## What the server's answer is checked for

A move or a rename makes the server tidy the imports of the files it edits, and the mirror has no libraries: Compose, Android and platform APIs do not resolve. An import used only through such a symbol looks unused and is deleted. The classic case is `import androidx.compose.runtime.getValue` and `setValue`, which `var clicks by remember { mutableStateOf(0) }` needs and never names (the fixture has exactly that and the test shows the server deleting them without this step). `mirror/imports.rs` takes the answer to `workspace/willRenameFiles` and to `textDocument/rename`, applies it to the file as it was, puts back every import line the server removed, and turns the result into edits again, one per changed run of tokens like the server's own, so that the rename engine can still match each edit to a place that refers to the symbol. The lines that may go are those the change calls for: for a move, the ones that point into the package a moved file left (the move rewrites those on purpose); for a rename, the ones that name the renamed symbol. When nothing was lost the server's edits are passed on untouched, and an answer that restores to nothing is dropped, so a file the change does not touch does not appear in the report.

## Limits

- **Files that declare `expect` or `actual` are not moved.** In the mirror an expect declaration and its actual are two declarations of one name in one module. The server refuses the second move of a pair ("Following declarations would clash"), and moving one side alone would leave the other in a package it must share. refac stops before the server starts and names the file and the line; move those files by hand (package line, and the compiler lists the imports that follow) and everything else with refac. A rename of a symbol that the named file declares `expect` or `actual` is refused the same way (``platformName` is declared `expect` …``): the rename would reach the pair through two declarations of one name. Rename those by hand and compile; the usages elsewhere are ordinary renames and work.
- **No semantics beyond the project's own classes.** Package lines and references to classes, functions and properties of the project follow, and so do uses inside the lambda of a library call (the fixture renames one inside `LaunchedEffect`). Names defined in two source sets of one package (an `actual class` in `androidMain` and in `desktopMain`) are one name to the server, and its edits do not tell the two apart. Compile both targets afterwards.
- **The Kotlin version of the mirror** comes from `REFAC_KOTLIN_MIRROR_VERSION`, else `kotlin = "x.y.z"` in `gradle/libs.versions.toml`, else the `kotlin("multiplatform") version "x.y.z"` line (or a `kotlin-gradle-plugin:x.y.z` classpath entry) of a build script. A version that cannot be read is an error naming the variable; a guess would not import.
- **The import needs the Kotlin Gradle plugin** of that version from the plugin portal or Maven Central, so a build that can only reach an internal repository needs the plugin in the Gradle cache of the machine.
- **Moves between source sets** (`commonMain` to `jvmMain`) are refused as before: no package edit can fix what the code is then allowed to see.
- **Not tested here:** a real Compose Multiplatform application with Android, desktop and iOS targets. The fixture `tests/fixtures/kotlin/kmp_project` has a common and a JVM target, an expect and its actual, a wildcard and an aliased import, a test source set, delegate imports and a stand-in for Compose that the mirror does not hold. Compile every target after a change: the compiler and the tests are the proof.

## Seeing what the server did

`RUST_LOG=debug refac move ...` prints the Gradle import output (`Gradle import: ...`), which is where `Failed to find 'target' in Kotlin extension` shows for a build the mirror did not cover; `REFAC_LSP_TRACE=full` prints every message exchanged with the server, in the mirror's paths.

## Names that stayed behind in the old package

A file sees every top-level name of its package without an import. When it moves to another package it needs imports for the ones that stayed. The server adds those it can resolve, but in the mirror an `expect fun` and its `actual fun` are two declarations of one name, which it takes for an ambiguous call: `platformName()` was left without an import and the real project failed to build with "Unresolved reference" (found 2026-10-10 by a colleague's script, reproduced with `shout(a) + platformName()` in a moved `commonMain` file; overloaded functions, extension functions and a second source set were imported correctly). `moves/same_package.rs` closes the gap for every Kotlin move, also outside Multiplatform: after the server, it adds `import <old package>.<name>` for every name a moved file uses that a file of the old package declares at the top level and the file does not import. See [File moves](file_Moves.md#imports-for-the-names-that-stayed-behind).

## Where it lives and how it is tested

- `src/drivers/kotlin/server/mirror.rs`: the mirror, URI translation, the file events, the refusal of expect/actual files and symbols. `mirror/layout.rs`: source roots, the multiplatform test, the Kotlin version. `mirror/imports.rs`: the kept imports and the edits. `mirror/modifiers.rs`: the blanked modifiers. `mirror/tests.rs`: unit tests without a server. `moves/same_package.rs` (with `moves/same_package/{used,declared,imports,scan_tests,tests}.rs`): the imports for the names that stayed behind.
- `tests/kotlin/multiplatform.rs` (real server, `#[ignore]`d except one): a two-file move across three source sets, the delegate imports, the expect/actual refusal, a symbol rename across source sets and inside a library call, the dry-run plan against the real move, the source-set refusal, and a moved file that used `shout` and the `expect fun platformName` of its old package without an import. Each success ends with `./gradlew compileKotlinJvm compileTestKotlinJvm`.
