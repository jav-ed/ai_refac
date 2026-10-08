# Kotlin Options

Which existing tools could give `refac` Kotlin support for Android work: moving Kotlin files and packages with their package declarations, imports, and usages, and renaming symbols. This document records what was checked on 2026-10-07, what the hands-on tests showed, and what is still unproven. The decision rule is to take the best existing engine instead of writing a Kotlin analyzer.

**Status: decided and implemented.** The recommendation below was validated hands-on on 2026-10-07 against build `ILS-263.6379.0` and became the Kotlin backend; the results are in the section "Hands-on results" at the end, and the resulting design is in [Kotlin and Android](../Features/Kotlin/linker_Kotlin.md).

## Why a semantic engine is needed

Kotlin does not require a file's directory to match its package, but conventions and Android tooling expect it. Moving a file to another package changes its `package` line and every import of its declarations. It also breaks usages that relied on the old package implicitly: code in the same package needs no import, so after the move those usages need new imports. A syntactic rewrite cannot know which names resolve to the moved file, so this needs name resolution.

## Candidates

| Candidate | Status | Rename | File and package move | Result |
|---|---|---|---|---|
| JetBrains Kotlin language server (`Kotlin/kotlin-lsp`) | Alpha, official, Apache-2.0 with some closed-source parts, last commit 2026-10-07, weekly builds, Linux/macOS/Windows archives | Listed feature | `workspace/willRenameFiles` handler runs IntelliJ's move processors (`K2MoveFilesHandler`, directory-with-classes helper); changelog says moving packages and several files at once improved | **Recommended.** Same protocol the Go and Dart drivers already use |
| `fwcd/kotlin-language-server` | Its README says it is deprecated in favour of the official server, last commit 2025-06-02 | Declaration-site variable rename only, from release notes | No file move | Rejected |
| OpenRewrite `rewrite-kotlin` | Recipe and visitor library for best-practice migrations, mirror last commit 2025-03-04. Kotlin sits on the Java tree, so `ChangeType` style recipes apply | Recipe-based | Recipe-based | Rejected for an interactive CLI: needs a build-tool run with a resolved classpath and writing recipes per operation |
| IntelliJ IDEA LSP extension | August 2026 preview for VS Code and Cursor, needs an IntelliJ IDEA install, subscription after the preview | Yes | Yes | Rejected: ties the CLI to a paid IDE install |
| JetBrains IDE MCP plugins (IDE Index MCP Server, MCP Code Intelligence) | Run inside a live JetBrains IDE, claim rename, move, and Android XML and manifest updates | Yes | Yes | Rejected as the engine: needs a running IDE. Their XML and manifest handling is a reference for what to test |
| Kotlin Analysis API (K2) standalone | Analysis only, no refactorings. It is what the language server builds on | No | No | Rejected: writing the refactoring layer is the work the official server already did |
| tree-sitter-kotlin or ast-grep | Syntactic | Text only | Text only | Rejected: cannot resolve names |

## What the source of `kotlin-lsp` shows

- `features-impl/kotlin/.../refactoring.xml` registers Kotlin's K2 move handlers (`K2MoveFilesHandler`, `K2MoveDirectoryWithClassesHelper`) inside the language server.
- `LSMoveFilesOrDirectoriesProcessor` wraps IntelliJ's `MoveFilesOrDirectoriesProcessor`: it finds usages, checks conflicts, and performs the move on an in-memory project.
- `refactoringUtils.kt` documents the `workspace/willRenameFiles` contract: IntelliJ simulates the whole refactoring and returns the text edits, while the rename or move of the file itself is left to the client and filtered out of the answer. The existing Go and Dart paths in `LspClient::initialize_and_rename_files` already apply exactly this contract.
- Rename goes through ordinary `textDocument/rename`, and a Java base module exists, so mixed Java and Kotlin projects are in scope.
- Build support: Gradle and Maven fully, Android Gradle Plugin experimental, Kotlin Multiplatform not yet. It is started with `kotlin-lsp.sh`; protocol options are listed by `--help`.

## Questions that were open before the hands-on tests

1. Does the standalone archive run headless over stdio with the installed JDK 24, and how long does Gradle import take on an Android module?
2. Does `willRenameFiles` rewrite the `package` line, imports, and same-package implicit usages correctly for a single file, several files, and a directory?
3. Does `textDocument/rename` cover properties, functions, classes, and Java callers, and does it detect name clashes? The TypeScript work showed that engines cannot be assumed to.
4. Android specifics: `R` references, view binding, layout XML that names a class in a tag, `AndroidManifest.xml` class names, and the Gradle `namespace`. These are the likely gaps if the Android plugin is not part of the server.
5. Memory and time on a real multi-module Android project, and how the helper's limits should apply.
6. Does the experimental Android Gradle Plugin import work on this machine's SDK?
7. Readiness: `src/drivers/lsp/client.rs` waits fixed sleeps (1.5 s) before asking for edits, and this already makes Dart moves report success without rewriting imports when the machine is loaded. A Gradle import is far slower, so the Kotlin backend needs a real readiness signal (progress notifications, or retrying until the server answers for a known file) and a hard error on timeout, not a reuse of the fixed sleep.

Answers: 1 yes (section below), 2 yes with the exceptions listed, 3 yes, but the server refuses only true redeclarations and accepts a shadowing rename, which refac catches, 4 as expected (the server never edits XML), 5 not measured on a large project, 6 yes (a real Android Gradle Plugin project imports and compiles), 7 solved with real signals.

## Hands-on results (2026-10-07)

Environment: build `ILS-263.6379.0` (Linux x64, 368,488,700 bytes, bundled JetBrains Runtime 25), JDK 21 for Gradle, Kotlin 2.4.20, Android Gradle Plugin 9.4.1, Gradle 8.14.3, Android SDK 36. Fixtures: a JVM project with Kotlin and a Java caller, and an Android application with manifest, layouts, a navigation graph, and a custom view. Every scenario ended with a Gradle compile.

### Starting and readiness

- It starts headless with `bin/intellij-server --stdio --system-path=<dir>` and asked for no EULA. `license status` reports that this build needs no license (EAP key valid through 2026-10-30). The bundled `EULA.txt` is missing, so `--show-eula` fails.
- `initialize` answers in about 3.5 seconds with `workspace.fileOperations.willRename` (glob `**/*`), a rename provider with `prepareProvider`, and incremental sync. A small Gradle project imports in about 10 seconds; `intellij/workspaceImportState` reports FINISHED at about 26 to 35 seconds and `intellij/ready-for-test` follows. Requests sent earlier are answered `null`. The server opens short "Indexing" progress tokens forever, so "no progress open" never settles; the two notifications above are the usable signal.
- A persistent `--system-path` gave no speedup (ready after about 29 seconds either way, the Gradle import dominates), so refac uses a temporary directory and deletes it (about 243 MB).

### File and package moves (`workspace/willRenameFiles`)

- One request takes one target directory. Several files into one directory, and a whole directory, work. Different target directories in one request, or a new directory plus a new name in one step, are answered `null` (a 60-second poll showed this is not a race). Refac therefore sends one request per target directory and splits a move with a new name into a move and a rename.
- Several groups in one session work when, after each group, the files are really moved, the edits are applied, and the server is told through `workspace/didChangeWatchedFiles` and `didOpen`/`didChange` of the moved and edited documents. Its file watcher is asynchronous; without this the next step answered `{}` (nothing to do) from stale state, which looks like success.
- Edits address the old paths (apply before the move) and sometimes a new path that does not exist yet.
- `.java` files and directories that contain Java are moved without updating their `package` lines. Refac refuses them. Kotlin moves update Java callers.
- Renaming a file in place also renames the class that carries the file's name. That is IDE behavior and is kept, checked, and reported.

### Symbol rename

- `textDocument/prepareRename`, `references`, and `rename` cover properties, functions, classes, members, and Java accessors of Kotlin properties. Edits are character-level diffs and positions are UTF-16.
- A true redeclaration (same signature in the same scope) is refused by the server itself. A **shadowing** rename is not refused: renaming an extension function to the name of a member of the same receiver produces edits that would move its call sites onto the member, and the server accepts them. After the new text is shown to the server, the references of the renamed declaration no longer include those call sites; comparing the references before and after is what refac uses (probe: a benign rename lost and gained nothing, Java sites included; the shadowing rename lost 2 call sites).
- Extension functions: the server lists no reference for the `import` line, yet rewrites it, or drops it (with the blank line after it) when the new name no longer needs it.
- A class rename returns a file `rename` operation after the text edits; refac applies it in order.
- Renaming a resource id is refused by the server, which is right.

### Android

- The server fixes `.kt` and `.java` sources and renames files with their class, but never edits XML: manifest `android:name` (fully qualified and relative `.Name`), custom-view tags and `tools:context` in layouts, and navigation `android:name` all keep the old class name. After a move out of the namespace package, the code lost the implicit `R` and `BuildConfig` (`Unresolved reference R`). Both are repaired by refac's Android layer; see [Android layer](../Features/Kotlin/android_Layer.md). A binding-property rename works.
- The Android Gradle Plugin import and `compileDebugKotlin` run on this SDK.

### Not measured or not tested

Cross-module moves (refused), Kotlin Multiplatform, Maven projects through refac's Android layer, a large multi-module Android project, and the server's memory on one. One run of the broken-build test hung for more than five minutes and could not be reproduced in eight further runs; the 600-second timeout and the import-log tail in the error are the guard.
