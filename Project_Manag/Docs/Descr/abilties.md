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
| **Rust** | `rust-analyzer` LSP plus embedded HIR | `rust-analyzer` binary for ordinary file renames |
| **Go** | `gopls` | `gopls` in PATH or `~/go/bin` |
| **Dart** | Dart SDK analysis server | `dart` (Dart SDK) |

Markdown-specific behavior, limits, and examples live in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md).

## 3. Known Limits Per Backend

| Language | Limit |
| :--- | :--- |
| **TypeScript / JS** | Complete caller updates require an authoritative `tsconfig.json` that includes all local TS/JS sources. Batches are limited to 30 contained source files. Details in [TypeScript Feature Docs](../Features/TypeScript/linker_TypeScript.md). |
| **Python** | Rope cannot trace imports that go through `__init__.py` re-exports (indirect imports). Rope is tried first; Pyrefly is the fallback. Details in [Python Feature Docs](../Features/Python/linker_Python.md). |
| **Markdown** | Details in [Markdown Feature Docs](../Features/Markdown/linker_Markdown.md). |
| **Rust** | Same-dir file renames use LSP symbol rename. Structural moves use `move-module`, move the complete conventional module subtree, rewrite resolved workspace references, never add `#[path]` shims, and roll back source changes when validation fails. Strict v1 rejects ambiguous or unsupported layouts. Details in [Rust Feature Docs](../Features/Rust/linker_Rust.md). |
| **Go** | Moving any file in a package renames the **entire package** (all `.go` files in that directory move together). Partial-package moves are not supported. A batch across N packages uses one gopls session total. Details in [Go Feature Docs](../Features/Go/linker_Go.md). |
| **Dart** | `.dart_tool/package_config.json` must exist in the project root for `package:` URI imports to be rewritten. Without it, only relative imports are updated. |

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
