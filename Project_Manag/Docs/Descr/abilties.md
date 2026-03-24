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
| **Python** | `Rope` / `Pyrefly` | `python3`, `rope`, `pyrefly` |
| **TypeScript / JS** | `Bun` | `bun` |
| **Markdown** | `Native Rust backend` | none |
| **Rust** | `rust-analyzer` | `rust-analyzer` binary |
| **Go** | `gopls` | `gopls` (Go Language Server) |
| **Dart** | `Dart SDK` | `dart language-server` |

Markdown-specific behavior, limits, and examples live in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md).

## 3. Known Limits Per Backend

| Language | Limit |
| :--- | :--- |
| **TypeScript / JS** | Projects >500 files skip cross-project reference updates on file moves. Details in [TypeScript Feature Docs](../Features/TypeScript/linker_TypeScript.md). |
| **Python** | Rope is tried first; Pyrefly is the fallback. Details in [Python Feature Docs](../Features/Python/linker_Python.md). |
| **Markdown** | Details in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md). |
| **Rust / Go / Dart** | No known limits beyond requiring the upstream tool to be installed and in PATH. |

## 4. Why Use `refac`?

Plain filesystem moves often break imports and module references. `refac` automates the follow-up updates so the project is more likely to remain buildable after structural changes.
