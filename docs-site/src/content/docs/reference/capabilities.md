---
title: Capabilities & Limits
description: Full summary of what refac supports across all languages.
---

## Core capabilities

| Capability | Detail |
|---|---|
| File moves | All supported languages |
| Directory moves | TypeScript / JavaScript only |
| Batch moves | All languages — mixed-language batches dispatched independently |
| JSON output | `--json` flag on any invocation |
| Human-readable output | Default |

## Language support matrix

| Language | Files | Directories | Engine |
|---|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ | ts-morph via Bun |
| Python | ✅ | ❌ | Rope (fallback: Pyrefly) |
| Rust | ✅ | ❌ | rust-analyzer (LSP) |
| Go | ✅ | ❌ | gopls (LSP) |
| Dart | ✅ | ❌ | Dart analysis server (LSP) |
| Markdown | ✅ | ❌ | Native Rust |

## Known limits per language

| Language | Key limit |
|---|---|
| TypeScript / JS | Projects > 500 source files skip cross-project reference updates on file moves. Point `--project-path` at a sub-package root to stay under the threshold. |
| Python | Rope cannot trace imports through `__init__.py` re-exports. Indirect callers are not updated. Namespace packages (no `__init__.py`) may see incomplete updates. |
| Rust | Cross-directory moves use a shim strategy (`#[path]` + `pub use` alias). Caller files are **not** rewritten. Same-directory renames fully update all `use` paths. |
| Go | Moving any `.go` file in a package moves the **entire package**. Partial-package moves are not supported. Requires `go.mod` for cross-directory moves. |
| Dart | `package:` URI imports only updated if `.dart_tool/package_config.json` exists. Run `dart pub get` to generate it. |
| Markdown | Only relative links rewritten. No directory moves. Non-`.md` files not scanned. |

## General limits (all languages)

- No dry-run mode — changes are applied immediately.
- No overwrite protection — existing target paths are overwritten.
- Build and vendor directories (`node_modules/`, `target/`, `.git/`, etc.) are excluded from reference scanning.
- Cross-crate (Rust) and cross-workspace reference updates are not supported.
