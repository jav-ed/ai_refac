# Capabilities & Supported Languages

`refac` is a CLI tool for moving and renaming files while updating affected references across a project.

## 1. Core Utilities

* **Intelligent Move/Rename**: Moves files and updates imports, module declarations, package references, or Markdown links where supported.
* **Dry run**: `refac move --dry-run` (every language), `refac move-module --dry-run` and `refac rename --dry-run` plan the operation, list the paths that would move and the edits per file, refuse what the real operation refuses, and write nothing. Backends that can read their tool's plan before it writes (TypeScript, Markdown, Dart, Go, Rust, Pyrefly) use it; Rope and the Kotlin server cannot, so a throw-away copy of the project takes the real move and the difference is reported.
* **Batch Operations**: Execute multiple move operations in one CLI invocation by repeating `--source-path` and `--target-path`.
* **Cross-Language Orchestration**: Routes each move to the correct backend for the target language.
* **Safety First**: Uses language-aware tooling instead of raw filesystem renames whenever possible.
* **Symbol Rename (TypeScript / JavaScript, Kotlin, Go, Rust, Python, Dart)**: `refac rename` renames a variable, parameter, function, class, interface, enum, method, or field and updates every reference. It plans and verifies the whole rename in memory, supports `--dry-run`, refuses name clashes and shadowing, and reports where the old name is still written. One command per rename, or `--batch` for several renames of one project in one language-server session (all or nothing; Kotlin, Go, Rust, Python, Dart): the language server is started and stopped by the command and never stays resident. Overview in [Symbol rename](../Features/Symbol_Rename/linker_Symbol_Rename.md); TypeScript and Kotlin also have [TypeScript symbol rename](../Features/TypeScript/symbol_Rename.md) and [Kotlin symbol rename](../Features/Kotlin/symbol_Rename.md).
* **Self-help for missing servers**: `refac doctor [language]` finds each language server, runs it once to prove it starts, and prints the install steps; every command that needs a missing server prints where it looked and names this command. See [Language servers](../Setup/language_Servers.md).
* **Android**: Kotlin moves and renames also update class names in the manifest, layouts, and navigation graphs, and add the `R` and `BuildConfig` imports a file loses when it leaves its namespace package. Details in [Android layer](../Features/Kotlin/android_Layer.md).
* **Human or JSON Output**: Supports human-readable output and machine-readable `--json` responses.

## 2. Currently Supported Languages

The tool integrates with the following language toolchains:

| Language | Driver Engine | Required Tooling |
| :--- | :--- | :--- |
| **Python** | `Rope` (primary) / `Pyrefly` (fallback) for moves; basedpyright for rename | `rope` package in `.venv` or `python3`; `pyrefly` only needed as fallback; `basedpyright` for `refac rename` |
| **TypeScript / JS** | Oxc parser + TypeScript resolver (moves); TypeScript 7 native language server (rename) | `bun` |
| **Markdown** | Native Rust backend (`pulldown-cmark` parser): Markdown files, assets, document folders, and links to files other backends moved | none |
| **Rust** | `rust-analyzer` LSP plus embedded HIR (moves); `rust-analyzer` (symbol rename) | `rust-analyzer` binary (`rustup component add rust-analyzer`) |
| **Go** | `gopls` (moves and symbol rename) | `gopls` (`REFAC_GOPLS`, `PATH`, `$GOBIN`, `$GOPATH/bin`, `~/go/bin`) |
| **Dart** | Dart SDK analysis server (moves and symbol rename) | `dart` (Dart SDK); symbol rename also needs `dart pub get` having run |
| **Kotlin / Android** | JetBrains Kotlin language server (moves and rename), refac's own Android XML layer | The server via `REFAC_KOTLIN_SERVER` ([setup](../Setup/kotlin_Server.md)), a JDK 17+, Gradle project; Android SDK for Android projects |

Markdown-specific behavior, limits, and examples live in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md).

## 3. Known Limits Per Backend

| Language | Limit |
| :--- | :--- |
| **TypeScript / JS** | Complete caller updates require an authoritative `tsconfig.json` that includes all local TS/JS sources. Batches are limited to 30 contained source files. Rename additionally needs a tsconfig that TypeScript 7 accepts (no `baseUrl`, no `node10` resolution). Details in [TypeScript Feature Docs](../Features/TypeScript/linker_TypeScript.md). |
| **Python** | Moves: Rope cannot trace imports that go through `__init__.py` re-exports (indirect imports). Rope is tried first; Pyrefly is the fallback. Rename: calls on untyped receivers and names in strings are reported, not renamed; special methods (`__init__`) are refused; a module name is changed with `refac move`. Details in [Python Feature Docs](../Features/Python/linker_Python.md) and [Python symbol rename](../Features/Symbol_Rename/python.md). |
| **Markdown** | Details in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md). |
| **Rust** | Same-dir file renames use LSP symbol rename. Structural moves use `move-module`, move the complete conventional module subtree, rewrite resolved workspace references, never add `#[path]` shims, and roll back source changes when validation fails. Strict v1 rejects ambiguous or unsupported layouts. Symbol rename does not reach into `macro_rules!` bodies (reported as an `ATTENTION` note) and refuses a module name (use `move-module`). Details in [Rust Feature Docs](../Features/Rust/linker_Rust.md) and [Rust symbol rename](../Features/Symbol_Rename/rust.md). |
| **Go** | Moving any file in a package renames the **entire package** (all `.go` files in that directory move together). Partial-package moves are not supported. A batch across N packages uses one gopls session total. Symbol rename refuses a package clause (use `refac move`). Details in [Go Feature Docs](../Features/Go/linker_Go.md) and [Go symbol rename](../Features/Symbol_Rename/go.md). |
| **Dart** | `.dart_tool/package_config.json` must exist in the project root for `package:` URI imports to be rewritten. Without it, a move that would leave a `package:` import dangling is refused before anything is written, with the imports listed. A symbol rename is refused with "Run `dart pub get`" for the same reason. Details in [Dart symbol rename](../Features/Symbol_Rename/dart.md). |
| **Kotlin / Android** | `--project-path` is the Gradle root. Every call imports the Gradle build first (about 30 seconds, about 1.6 GiB for the server on a tiny project), so batch moves into one call. Directory moves work; `.java` files, directories containing Java, and moves between modules or source sets are refused. Old class names in ProGuard rules, build scripts, and string literals are reported, not rewritten. Details in [Kotlin Feature Docs](../Features/Kotlin/linker_Kotlin.md). |

## 4. JSON Output

Pass `--json` to get machine-readable output instead of human-readable terminal text. Returns a single JSON object:

```json
{ "status": "ok", "operation": "move", "result": "..." }
```

On partial or full failure:

```json
{ "status": "error", "error": "..." }
```

`"error"` on failure contains the descriptive error chain. Exit codes: `0` = all succeeded, `1` = one or more failed.

The `--json` flag is the intended interface for agent use. Parse `status` to branch, then read the operation-specific success fields or `error` for detail.

## 5. Why Use `refac`?

Plain filesystem moves often break imports and module references. `refac` automates the follow-up updates so the project is more likely to remain buildable after structural changes.
