# Kotlin and Android

Read this file before the first Kotlin or Android move or rename in a session, or when one fails or leaves the build broken. `move` and `rename` on `.kt` files use the JetBrains Kotlin language server; refac adds layers around it for what the server leaves out. `--project-path` is the **Gradle root** (the folder with `settings.gradle(.kts)`), not a module folder.

## The common case

```bash
# Several moves in ONE call (a single Kotlin move is refused by default: every call costs ~24 s)
refac move --project-path /path/to/gradle-root --dry-run --batch moves.json   # check the list
refac move --project-path /path/to/gradle-root --batch moves.json             # run it
```

`moves.json` is `[{"source_path": "app/src/main/kotlin/com/x/ui/Home.kt", "target_path": "app/src/main/kotlin/com/x/home/Home.kt"}, ...]`, exactly those two keys per entry (`--batch -` reads stdin). The flags `--source-path` / `--target-path` do the same. Then run the project's build (`./gradlew compileKotlin`, and every target of a multiplatform build): refac proves the package lines, not the compile.

After a move read the `// Note:` lines. They name the files whose content the move changes (`This move changes the content of N files: ...`; a change in any other file is not from the move), the imports refac added, old class names that still appear in files it does not edit, and the multiplatform and cost notes.

## What a move does

- **Accepts** a `.kt` file or a directory with `.kt` files, inside `src/<set>/kotlin` or `src/<set>/java`. The `package` line, imports and usages that relied on sharing a package are updated, Java callers too. A new directory plus a new name is done as move then rename; renaming a file also renames its class.
- **Changes only what a move has to.** The package line and the imports. Unused imports of things that stayed are kept, a change of layout alone (a call the server wrapped over four lines) is undone, and so is an operator the server rewrote as a call (`a < b` as `a compareTo b`). Files that `.gitignore` leaves out (reference clones, scratch folders) are not read. A file that is not valid UTF-8 does not fail the move; a note names it if it mentions an old class name.
- **Imports what the moved file loses.** A file sees every top-level name of its package without an import. When it leaves, the server adds the imports it can resolve and refac adds the ones it leaves out: overloads, an `expect` function with its `actual`, and operator functions that the syntax calls without writing their names (`q[0]` calls `get`, `for (x in q)` calls `iterator`, `val (a, b) = q` calls `component1` and `component2`, `val n by registry` calls `getValue`/`setValue`). The server imports the operators written as symbols (`+`, `<`, `in`). The other way round, files that stayed behind get an import of the operator functions that moved away with their declaring file.
- **Limits of those imports.** Operator imports are made only when the file writes the type the extension is called on (`Qty` for `fun Qty.get`), and names of another Gradle module only for a module the build file names as a project dependency (`project(":lib")` or `projects.lib`, direct dependencies; `api` chains are not followed). Names of other modules are otherwise the server's job. Compile afterwards.

## When it fails

A failed move puts every file back and prints `Kotlin — The move failed and every change was undone: Moving a -> b: <reason>`. The reason is the whole chain of causes. Tell the user that line, whole:

| The reason says | What it means |
|---|---|
| `The Kotlin language server (...) closed its connection ...` followed by `The process was killed by signal 9` | The server was killed, usually by the Linux out-of-memory killer (it holds 1.3 to 1.8 GB, more on a big project). Free memory or swap, then retry. `dmesg` or `journalctl -k` names the kill. |
| the same, with `exited with status N` and `Its last lines on stderr:` | The server crashed; the stderr lines are its own explanation (a JVM error, a missing JDK 17+). |
| `Last Gradle import output:` | The Gradle import of the project failed; the lines name the broken build file or plugin. Fix the build first. |
| `did not answer ... within N seconds` / `did not signal readiness` | Too slow; raise `REFAC_KOTLIN_TIMEOUT_SECS` (default 600). |
| `refused the move` | The server declined the move; the text is its reason. |

## Refused before anything starts

`.java` files, directories that contain Java, moves between modules or source sets, a target that exists, a target outside a source root, and a file that declares `expect` or `actual` (move it by hand). A single Kotlin move or rename is refused with the batch command written out and the cost explained (`refac guide kotlin`); `--allow-single` lets one through.

## Multiplatform

A Kotlin Multiplatform build (a `commonMain`, `commonTest` or `<target>Main` source set) is served through a plain-JVM copy of its source sets: package lines and project imports follow, library imports are kept, and the output ends with a note to compile every target.

## Install, rename, Android, notes

- **Install:** `refac doctor kotlin` prints the steps and `REFAC_KOTLIN_SERVER` (a download of about 350 MB, 1.2 GB unpacked, JDK 17+). Without the server every Kotlin call fails with those steps; refac never downloads it. It keeps an index of the JDK and Kotlin libraries in `~/.cache/refac` (110 to 160 MB); `REFAC_KOTLIN_CACHE=off` keeps nothing.
- **Rename:** pick the symbol with `--file` and `--symbol`; an ambiguous name lists candidates for `--line`/`--column`. A rename that would clash with or shadow another declaration is refused. Renaming a class that names its file renames the file too. `--dry-run` writes nothing. Several renames: `rename --batch`.
- **Android:** class names in the manifest, layouts and navigation graphs follow moved and renamed classes, and `import <namespace>.R` / `BuildConfig` is added where a moved file needs it. A module with a manifest but no `namespace` in its build script is an error.
- **Reported, not edited:** old class names in ProGuard rules, build scripts (`mainClass`), service lists, configuration and string literals appear as `// Note:` lines. Search for them and run the build.
