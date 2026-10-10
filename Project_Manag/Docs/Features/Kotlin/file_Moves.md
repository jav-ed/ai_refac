# Kotlin File Moves

`refac move` on `.kt` files, or on a directory that contains `.kt` files, moves them and updates the `package` line, every import, and the usages that relied on sharing a package (code in the same package needed no import, so after a move those usages gain one). Java callers of the moved Kotlin classes are updated too. The Kotlin language server computes all text edits by running IntelliJ's own move refactoring in memory; refac then applies them together with the file moves.

```bash
refac move --project-path /path/to/gradle/root \
  --source-path app/src/main/kotlin/com/example/util/Helper.kt \
  --target-path app/src/main/kotlin/com/example/common/Helper.kt
```

## What a request may be

- **Files:** `.kt` source or a directory, inside a source root `src/<source set>/kotlin` or `src/<source set>/java`. The server derives the new package from the target directory, so a target outside a source root is refused.
- **Move, rename, or both:** a pure move keeps the file name, a rename keeps the directory, and a move to a new directory with a new name is done as two steps (move, then rename) because the server cannot do both at once. The output says when that happened. A temporary name that is already taken is refused with a request to split the call.
- **Renaming a file renames its class:** renaming `Helper.kt` to `Utils.kt` also renames the top-level type `Helper` to `Utils` when the file declares a type named like the file, and every usage follows. This is how the IDE behaves. It also applies to the rename step of a move with a new name. Refac checks that it happened and says so in the notes. Files without a type of that name are only renamed.
- **Directories:** a package directory moves with its files and every subpackage.
- **Several requests in one call:** requests are grouped by kind and target directory, one server request per group, all inside one server session. Requests must not overlap: no source inside another, no duplicate target, no target that is another request's source.

## Refusals before anything starts

Each of these fails before the server is started and changes nothing:

- A `.java` file, or a directory that contains `.java` files. The server moves Java files without updating their package line, so refac refuses instead of leaving the project inconsistent. Move the Kotlin files, or use a Java-aware tool for Java.
- A source or target that is not inside a source root, or is a source root itself.
- A source and a target in different modules or source sets (`src/main` versus `src/test`, `commonMain` versus `jvmMain`): no package edit can fix what the code is then allowed to see. In a Kotlin Multiplatform build the server is served a plain-JVM copy of the source sets; see [Kotlin Multiplatform](multiplatform_Mirror.md).
- A target that already exists, a target that keeps no `.kt` extension, a `..` in a target path, a move into itself.

## Checks after the server answers

The server is trusted for the reference updates, and refac checks what is cheap to check. A moved file's `package` line must now follow its directory (relative to the source root), and a file renamed in place must have taken the class that carried its name. A server answer that does nothing (it answers `null` or `{}` when it is not ready or not able) shows up here as a failed check and not as a quiet success. A file whose package line did not follow its directory before the move cannot be compared; that is reported in the notes as not checked.

## All or nothing

Every write and every move goes through an undo log. When anything fails, whether a server error, a failed check, or a write error, the log restores every edited file and every moved path and the error says that the move was undone. A rollback that itself fails is reported with the failure, never hidden.

## A single file says what it cost

A Kotlin command pays the server start and the Gradle import (about 45 seconds the first time and 20 after that, 1.3 to 1.8 GB) whatever the number of files, and each command is a process of its own, so refac cannot tell that it is the twentieth. `src/logic/kotlin_cost.rs` therefore acts on what one command can see. A move of exactly one Kotlin file (a folder, several files and a batch are left alone) is an error before anything starts (`refuse_single_move`, dry runs included, one Kotlin file next to files of other languages still counts), with the `refac move` line for that request written out. The refusal is the default; `--allow-single` lets the one move through and `REFAC_KOTLIN_BATCH_ONLY=0` turns the refusal off for the whole environment. A move that is let through ends with a note that gives the seconds it took and the batch form; in a dry run the note adds that the real run starts the server again. The rename does the same for a single Kotlin rename (`refuse_single_rename`, the note in `handle_rename`), with a ready `rename --batch -` line. The variable is `1` (also when empty or unset) or `0`; anything else is an error. The test helpers in `tests/common` set it to `0` because they move single Kotlin files on purpose. Tests: `tests/cli/kotlin_cost.rs` (no server), `tests/kotlin/dispatch.rs` and `tests/kotlin/dry_run.rs` (the notes on real runs).

## Dry run (`move --dry-run`)

The moves run in groups, each asked of the server after the one before was written, and then the Android layer reads the moved files, so there is no plan to read before the first write. The preview runs the real move on a throw-away copy of the project and reports the difference to the original (the moves, and per file the number of changed passages). The copy holds what `.gitignore` leaves in (plus `local.properties` and `gradle/wrapper`), at most 500 MiB (`REFAC_DRY_RUN_COPY_MAX_MB`); `.gradle`, `.kotlin` and the `build` output the import creates are not part of the answer. The server imports the copy with Gradle, so a dry run takes as long as a real move (about 40 seconds on a small project). Every path must lie inside the project.

## What the output tells you

Besides the list of moves, `refac move` prints `// Note:` lines for things only the caller can act on:

- the move and rename split above,
- Android class names that were rewritten in XML and `R`/`BuildConfig` imports that were added (see [Android layer](android_Layer.md)),
- old class names that still appear in files refac does not edit, with the file, the old name, and the new name, for example `build.gradle.kts still mentions com.example.app.MainKt (now com.example.launch.MainKt)`. Search for these and fix them by hand.

After a move, run the project's Gradle build. The tests of this repo do exactly that after every scenario.
