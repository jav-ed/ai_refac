# Kotlin Options

Which existing tools could give `refac` Kotlin support for Android work: moving Kotlin files and packages with their package declarations, imports, and usages, and renaming symbols. This document records what was checked on 2026-10-07 and what is still unproven. The decision rule is to take the best existing engine instead of writing a Kotlin analyzer.

**Status: paused on 2026-10-07 at the start of hands-on validation.** The conclusions below are source-level and documentation evidence, so they are a recommendation, not a verified result. The standalone server is downloaded to `~/Progs/kotlin-lsp/` and the first protocol checks are in the section "Hands-on progress" at the end.

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

## Open questions for hands-on testing

1. Does the standalone archive run headless over stdio with the installed JDK 24, and how long does Gradle import take on an Android module?
2. Does `willRenameFiles` rewrite the `package` line, imports, and same-package implicit usages correctly for a single file, several files, and a directory?
3. Does `textDocument/rename` cover properties, functions, classes, and Java callers, and does it detect name clashes? The TypeScript work showed that engines cannot be assumed to.
4. Android specifics: `R` references, view binding, layout XML that names a class in a tag, `AndroidManifest.xml` class names, and the Gradle `namespace`. These are the likely gaps if the Android plugin is not part of the server.
5. Memory and time on a real multi-module Android project, and how the helper's limits should apply.
6. Does the experimental Android Gradle Plugin import work on this machine's SDK?
7. Readiness: `src/drivers/lsp_client.rs` waits fixed sleeps (1.5 s) before asking for edits, and this already makes Dart moves report success without rewriting imports when the machine is loaded. A Gradle import is far slower, so the Kotlin backend needs a real readiness signal (progress notifications, or retrying until the server answers for a known file) and a hard error on timeout, not a reuse of the fixed sleep.

The test fixtures, the pass and fail results, and the final decision belong in this document once the archive is available.

## Hands-on progress (paused 2026-10-07)

Verified on build `ILS-263.6379.0` (Linux x64, 368,488,700 bytes, bundled JetBrains Runtime 25):

- It starts with `bin/intellij-server --stdio --system-path=<dir>` and did not ask for EULA acceptance. `license status` reports that this build needs no license (EAP key valid through 2026-10-30), which matters if a later build changes that. The bundled `EULA.txt` is missing, so `--show-eula` fails.
- `initialize` answers in about 3.5 s with `workspace.fileOperations.willRename` (glob `**/*`), a rename provider with `prepareProvider`, and incremental text sync.
- A small Gradle project imports in about 10 s; the import-finished state arrives at about 26 to 35 s. The server sends `intellij/importLog`, `intellij/workspaceImportState` and `intellij/ready-for-test`, which are usable readiness signals. It keeps opening short "Indexing" progress tokens indefinitely, so a "no progress open" check never settles.
- Not yet run: the actual move and rename scenarios, the Android fixture, and the memory and time measurements. The test harness and fixture are in `Scratch/Kotlin_Lsp_Trial/` (gitignored) and the next steps are in the PAUSED section of `Scratch/Agent_Tasks/ts_variable_rename_24a2a9b7.md`.
