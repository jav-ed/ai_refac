# Kotlin File Moves

`refac move` on `.kt` files, or on a directory that contains `.kt` files, moves them and updates the `package` line, every import, and the usages that relied on sharing a package (code in the same package needed no import, so after a move those usages gain one). Java callers of the moved Kotlin classes are updated too. The Kotlin language server computes all text edits by running IntelliJ's own move refactoring in memory; refac then applies them together with the file moves.

```bash
refac move --project-path /path/to/gradle/root \
  --source-path app/src/main/kotlin/com/example/util/Helper.kt \
  --target-path app/src/main/kotlin/com/example/common/Helper.kt
```

## What a request may be

- **Files:** `.kt` source or a directory, inside a source root `src/<source set>/kotlin` or `src/<source set>/java`. The server derives the new package from the target directory, so a target outside a source root is refused.
- **Move, rename, or both:** a pure move keeps the file name, a rename keeps the directory, and a move to a new directory with a new name is done as two steps (move, then rename) because the server cannot do both at once (a folder too: `util` to `lib/helpers`). The output says when that happened. A temporary name that is already taken is refused with a request to split the call.
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

## Imports for the names that stayed behind

A Kotlin file sees every top-level declaration of its own package without an import, so a file that moves to another package loses them. The server adds the imports it can resolve; it leaves out names it takes for ambiguous, and in the Multiplatform mirror that includes an `expect fun` with its `actual fun` (see [the mirror](multiplatform_Mirror.md#names-that-stayed-behind-in-the-old-package)). `src/drivers/kotlin/moves/same_package.rs` runs after the server and adds `import <old package>.<name>` for each name that

- the moved file uses outside comments and strings (inside `$name` and `${...}` templates too; after a dot only for extension functions and properties),
- a file of the old package declares at the top level, not `private`: Kotlin files by their column-0 declarations (functions, properties, classes, objects, typealiases, with modifiers and annotations on the same line), Java files by the class that carries the file name,
- and the file does not import already, under that simple name (an explicit import of another `Helper`, or an alias, is left alone: a second one would clash) or by a star import of the old package.

A name is only taken from the same Gradle module and from a source set the moved file can see (its own, `main`, `commonMain`; a test set also sees `commonTest` and its platform's main set). A type that a moved file declares itself is not imported. Each file that got imports is named in the notes, so the additions can be checked. It looks at names, so overloads, `expect` and `actual` are no problem.

### Operators and delegates: calls that need an import without writing the name

The server imports the operators that are written as symbols (`+ - * / % ! ++ -- < > <= >= in .. += invoke`, tested against the real server), and it misses the ones that the syntax calls: `a[i]` and `a[i] = v` (`get`, `set`), `for (x in y)` (`iterator`, `hasNext`, `next`), `val (a, b) = p` and `{ (a, b) -> }` (`componentN`), `val n by d` (`getValue`, `setValue`, `provideDelegate`). Without the import the build says "No 'get' operator method providing array access". `same_package/convention.rs` finds those uses in the moved file (comments and strings are skipped; `for (x in y)` is told from a destructuring by its parenthesis), and `same_package.rs` adds the import when

- the declaration is an `operator` extension of the old package (`Declaration.operator`, read by `declarations/top_level.rs`, which also records the receiver type), and
- the file writes the receiver type (`Qty` for `operator fun Qty.get`; a type-parameter receiver matches any file). That is the precision rule: `list[0]` or `by lazy` in a file does not import an unrelated `get` or `getValue` of the package.

The server also gets some operators wrong, which is why it is not trusted for them either: it rewrites `a < b` as `a compareTo b`, `x in y` as `y contains x` and `a[i] = v` as the infix call `a set i v` when it adds the import, and the infix call does not compile for a non-infix function. `server/mirror/operators.rs` undoes that by token diff (`undo_operator_calls`, before `undo_reformatting`), keeping the import and the written syntax.

The files that stay behind are served by `same_package/users.rs`: when a moved file declares an `operator` extension, the files of the old package (and the files that star-import it) that use it through the syntax get the import, because the declaration left the package they shared. A note names each file that got imports.

**Other Gradle modules.** A plain name of another module is the server's job and stays so. For an operator, `same_package/dependency.rs` reads the module's build file (`build.gradle.kts` or `build.gradle`) for `project(":lib")` or `projects.lib`, whatever the configuration around it, and takes candidates from the modules it names, direct dependencies only (`api` chains are not followed: a module reached through another module's `api` is not seen). Tests: `same_package/tests/operators.rs` and `dependency_tests.rs` (plan level), `tests/kotlin/moves.rs` with the fixtures `jvm_project/.../ops/` and `multi_module/` (real server and compile).

## A move changes the package line and the imports, nothing else

The server tidies the files it edits: it deletes imports it takes for unused and it can re-wrap code. A move is not for that, so refac undoes it (`src/drivers/kotlin/server/mirror/{imports,restore,untouched}.rs`, for every Kotlin project; the details are in [the mirror's page](multiplatform_Mirror.md#what-the-servers-answer-is-checked-for)):

- an unused import stays, unless it names a class or function that a moved file declared. That import is rewritten to the new package, as for a used one, also when the server deleted it;
- a change that only moves blanks and line breaks below the import lines is undone;
- what refac does add is the missing imports of the section above.

A rename of a file also renames its class; an unused import of that class is left to the server.

## What refac reads of the rest of the project

After the server has answered, three layers read files of the project: the Android layer (manifest, layouts), the scan for old class names (ProGuard rules, build scripts, configuration, string literals) and the imports for the names that stayed behind. They read what `.gitignore` leaves in, like the Markdown pass: a reference clone or a scratch folder in the same repository is not the project. The scan for old names only reports, so a file it cannot read, or that is not UTF-8 (a data file in Windows-1256), does not undo a move: it is read as far as it is text, or named in a note ("X could not be read (...), so refac did not check it for old class names"). Until 2026-10-12 one such file anywhere under the Gradle root made every Kotlin move of that project fail after the server was done, and the report said only "The move failed and every change was undone".

## All or nothing

Every write and every move goes through an undo log. When anything fails, whether a server error, a failed check, or a write error, the log restores every edited file and every moved path and the error says that the move was undone, followed by the cause (`Kotlin — The move failed and every change was undone: Moving a -> b: <reason>`; the whole chain of causes is printed, not only the outer line). A rollback that itself fails is reported with the failure, never hidden. When the language server itself dies (an out-of-memory kill, a JVM crash), the cause names how the process ended (the exit status, or the signal: 9 is usually the Linux out-of-memory killer) and its last lines on stderr; `LspSession` keeps the stderr of every server in a temporary file for that (`src/drivers/lsp/session/exit.rs`), so the reason is in the first report and nobody has to ask for it.

## A single file says what it cost

A Kotlin command pays the server start and the Gradle import (about 44 seconds the first time and 24 after that, 15 with `REFAC_KOTLIN_GRADLE_IDLE_SECS` and a call right before, 1.3 to 1.8 GB) whatever the number of files, and each command is a process of its own, so refac cannot tell that it is the twentieth. `src/logic/kotlin_cost.rs` therefore acts on what one command can see. A move of exactly one Kotlin file (a folder, several files and a batch are left alone) is an error before anything starts (`refuse_single_move`, dry runs included, one Kotlin file next to files of other languages still counts), with the `refac move` line for that request written out, and the same moves as a JSON list for `refac move --batch -` (a list in a file works as well; see `src/cli/batch.rs`). The refusal is the default; `--allow-single` lets the one move through and `REFAC_KOTLIN_BATCH_ONLY=0` turns the refusal off for the whole environment. A move that is let through ends with a note that gives the seconds it took and the batch form; in a dry run the note adds that the real run starts the server again. The rename does the same for a single Kotlin rename (`refuse_single_rename`, the note in `handle_rename`), with a ready `rename --batch -` line. The refusal prints the long explanation (`src/logic/kotlin_cost/explanation.txt`, shown by `refac guide kotlin`). The variable is `1` (also when empty or unset) or `0`; anything else is an error. The test helpers in `tests/common` set it to `0` because they move single Kotlin files on purpose. Tests: `tests/cli/kotlin_cost.rs` (no server), `tests/kotlin/dispatch.rs` and `tests/kotlin/dry_run.rs` (the notes on real runs).

## Dry run (`move --dry-run`)

The moves run in groups, each asked of the server after the one before was written, and then the Android layer reads the moved files, so there is no plan to read before the first write. The preview runs the real move on a throw-away copy of the project and reports the difference to the original (the moves, and per file the number of changed passages). The copy holds what `.gitignore` leaves in (plus `local.properties` and `gradle/wrapper`), at most 500 MiB (`REFAC_DRY_RUN_COPY_MAX_MB`); `.gradle`, `.kotlin` and the `build` output the import creates are not part of the answer. The server imports the copy with Gradle, so a dry run takes as long as a real move (about 40 seconds on a small project). Every path must lie inside the project.

## What the output tells you

Besides the list of moves, `refac move` prints `// Note:` lines for things only the caller can act on:

- the move and rename split above,
- Android class names that were rewritten in XML and `R`/`BuildConfig` imports that were added (see [Android layer](android_Layer.md)),
- the files whose content the move changes: `This move changes the content of N files: a, b, ...` (project-relative, at most 40 named, then "and N more"). The note is the answer to "what did refac touch": a change in any other file of `git diff` is not from the move. A dry run prints the same note (`moves/edited.rs`),
- old class names that still appear in files refac does not edit, with the file, the old name, and the new name, for example `build.gradle.kts still mentions com.example.app.MainKt (now com.example.launch.MainKt)`. Search for these and fix them by hand.

After a move, run the project's Gradle build. The tests of this repo do exactly that after every scenario.
