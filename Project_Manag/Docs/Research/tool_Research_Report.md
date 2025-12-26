# Comprehensive Refactoring Tools Research Report

## Objective

Identify the best backend tools for a Rust-based MCP Refactoring Server capable of symbol renaming and file moves across multiple languages.

---

## 1. TypeScript / JavaScript

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **ts-morph** | Library | ✅ | ✅ | Moderate | 10/10 |
| **oxc** | Rust CLI/Lib | ⚠️ | ✅ (via linter) | High | 9/10 |
| **ast-grep** | CLI | ⚠️ | ❌ | High | 8/10 |
| **biome** | CLI | ❌ | ❌ | High | 7/10 |
| **putout** | CLI/Lib | ✅ | ❌ | Moderate | 7/10 |
| **recast** | Library | ⚠️ | ❌ | Moderate | 6/10 |

### Top Recommendation: **ts-morph**

- **Pros**: Most reliable for project-wide symbol renaming and automatic import updates. Handles `tsconfig.json` perfectly.
- **Cons**: Requires a Node.js process.
- **Use Case**: Deep, semantic refactoring that must not break the project.

### Performance Choice: **oxc**

- **Pros**: Written in Rust, extremely fast. `oxlint` can handle many automated fixes.
- **Cons**: Less focused on "symbol rename" across the whole project graph compared to `ts-morph`.

---

## 2. Python

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **rope** | Library/CLI | ✅ | ✅ | Moderate | 9/10 |
| **Pyrefly** | LSP/Typed | ✅ | ✅ | High | 9/10 |
| **libcst** | Library | ✅ | ✅ | Moderate | 8/10 |
| **pyupgrade** | CLI | ❌ | ❌ | High | 7/10 |
| **griffe** | Library | ❌ | ❌ | High | 6/10 |

### Top Recommendation: **rope** (via `ropecli`)

- **Pros**: The gold standard for Python refactoring. Safe, handles complex moves/renames.
- **Cons**: Library-first; CLI output needs parsing.

### Modern Recommendation: **Pyrefly**

- **Pros**: Extremely fast (1M+ LoC/sec). Handles automatic import refactoring on move/rename. Native support for large-scale Meta-level codebases.
- **Cons**: Newer, focused on type-checking and LSP features.

---

## 3. Rust

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **rust-analyzer** | LSP | ✅ | ✅ | High | 9/10 |
| **IntelliJ Rust** | IDE/Engine | ✅ | ✅ | High | 9/10 |
| **syn / quote** | Library | ⚠️ | ❌ | High | 5/10 (Low-level) |
| **cargo fix** | CLI | ❌ | ❌ | High | 3/10 |

### Top Recommendation: **rust-analyzer** (Headless LSP)

- **Pros**: Deepest semantic understanding. Supports atomic renames and "Assists".
- **Cons**: No standalone "refactor" CLI; must communicate via JSON-RPC.

---

## 4. Go

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **gopls** | LSP | ✅ | ✅ | High | 10/10 |
| **gopatch** | CLI | ⚠️ | ❌ | High | 8/10 |
| **gorename** | CLI | ✅ | ❌ | Moderate | 6/10 (Obsolete) |
| **gomove** | CLI | ❌ | ✅ | Moderate | 6/10 |

### Top Recommendation: **gopls** (Official Language Server)

- **Pros**: The industry standard. Supports type-safe renames and complex refactorings (extract/inline) via CLI `gopls rename` and `gopls codeaction`.
- **Cons**: Most robust when used as a live LSP, but CLI mode is powerful for headless tasks.
- **Use Case**: All semantic Go refactoring. Use `gopatch` for structural transformations.

---

## 5. Dart

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **Analysis Server** | LSP | ✅ | ✅ | High | 9/10 |
| **dart fix** | CLI | ⚠️ | ❌ | High | 8/10 |
| **mass_refactor** | CLI | ⚠️ | ✅ (Basic) | Moderate | 6/10 |

### Top Recommendation: **Dart Analysis Server** (Headless LSP)

- **Pros**: Powers all Dart IDEs. Handles intelligent symbol renames and file moves with high precision.
- **Cons**: Requires JSON-RPC/LSP communication for most operations. No simple "rename" CLI subcommand.
- **Use Case**: Deep semantic refactoring. Use `dart fix` for automated linting/API migrations.

---

## 6. Multi-Language / Structural

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **semgrep** | CLI | ⚠️ | ❌ | High | 8/10 |
| **ast-grep** | CLI | ⚠️ | ❌ | High | 8/10 |
| **comby** | CLI | ⚠️ | ❌ | High | 7/10 |

- **semgrep**: Best for large-scale rule-based migrations (e.g., API changes).
- **comby**: Fastest for language-agnostic structural search/replace.

---

## 7. Rust-Based Refactoring Engines (Cross-Language)

For a Rust-based MCP server, leveraging Rust-native engines provides the best performance and easiest integration (via FFI or direct crate usage).

| Language | Top Rust Option | Status | Capability |
| :--- | :--- | :--- | :--- |
| **JS/TS** | **oxc** | Production-ready | High-speed AST transformation and lint-fixing. |
| **Python** | **Pyrefly** | Production-ready | High-speed type checking and import refactoring. |
| **Multi** | **ast-grep** | Production-ready | Sgrep-like structural search and rewrite in Rust. |
| **Go** | **go_parser** / **Gold** | Library / Linter | AST parsing in Rust; full refactor engine requires wrapping. |
| **Dart** | **oak-dart** | Library | High-performance Dart parsing and AST generation in Rust. |
| **Foundation** | **tree-sitter** | Universal | The "gold standard" for incremental parsing used by these engines. |

### Architectural Recommendation for Rust MCP Server

- **Native Drivers**: Use `oxc` and `Pyrefly` directly for maximum throughput in JS/Python.
- **LSP Drivers**: Use `rust-analyzer` (Rust), `gopls` (Go), and `Analysis Server` (Dart) as "Semantic Oracles" when deep symbol resolution across modules is required.
- **Pattern Drivers**: Use `ast-grep` for simpler, language-agnostic structural transformations.

---

## Final Synthesis for MCP Refactoring Server

For a Rust-based MCP Refactoring Server, the ideal architecture would be:

1. **TypeScript**: Wrap `ts-morph` in a lightweight Node script invoked by Rust.
2. **Python**: Use `ropecli` for deep refactors or `pyrefly` for performance.
3. **Rust**: Use `rust-analyzer` in headless mode via JSON-RPC.
4. **General Patterns**: Use `ast-grep` or `semgrep` for simpler, pattern-based changes.
