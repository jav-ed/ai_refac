# Capabilities & Supported Languages

`refac` is a CLI tool for moving and renaming files while updating affected references across a project.

## 1. Core Utilities

* **Intelligent Move/Rename**: Moves files and updates imports, module declarations, package references, or Markdown links where supported.
* **Batch Operations**: Execute multiple move operations in one CLI invocation by repeating `--source-path` and `--target-path`.
* **Cross-Language Orchestration**: Routes each move to the correct backend for the target language.
* **Safety First**: Uses language-aware tooling instead of raw filesystem renames whenever possible.
* **Human or JSON Output**: Supports human-readable output and machine-readable `--json` responses.

## 2. Currently Supported Languages

The tool integrates with the following language toolchains:

| Language | Driver Engine | Required Tooling |
| :--- | :--- | :--- |
| **Python** | `Rope` (primary) / `Pyrefly` (fallback) | `rope` package in `.venv` or `python3`; `pyrefly` only needed as fallback |
| **TypeScript / JS** | `Bun` | `bun` |
| **Markdown** | Native Rust backend | none |
| **Rust** | `rust-analyzer` | `rust-analyzer` binary |
| **Go** | `gopls` | `gopls` in PATH or `~/go/bin` |
| **Dart** | Dart SDK analysis server | `dart` (Dart SDK) |

Markdown-specific behavior, limits, and examples live in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md).

## 3. Known Limits Per Backend

| Language | Limit |
| :--- | :--- |
| **TypeScript / JS** | Projects >500 files skip cross-project reference updates on file moves. Details in [TypeScript Feature Docs](../Features/TypeScript/linker_TypeScript.md). |
| **Python** | Rope cannot trace imports that go through `__init__.py` re-exports (indirect imports). Rope is tried first; Pyrefly is the fallback. Details in [Python Feature Docs](../Features/Python/linker_Python.md). |
| **Markdown** | Details in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md). |
| **Rust** | Uses a shim strategy for cross-directory moves: a `#[path]` attribute is added in the declaring file and a `pub use` alias is created, so caller files are left unchanged. No direct import rewriting. |
| **Go** | Moving any file in a package renames the **entire package** (all files in that directory move together). This is gopls behaviour — Go's package-per-directory model does not support partial-package moves. |
| **Dart** | `.dart_tool/package_config.json` must exist in the project root for `package:` URI imports to be rewritten. Without it, only relative imports are updated. |

## 4. Why Use `refac`?

Plain filesystem moves often break imports and module references. `refac` automates the follow-up updates so the project is more likely to remain buildable after structural changes.
