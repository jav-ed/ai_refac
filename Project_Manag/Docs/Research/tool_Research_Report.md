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

## 4. Multi-Language / Structural

| Tool | Type | Rename | Move File | Performance | Rating |
| :--- | :--- | :---: | :---: | :--- | :---: |
| **semgrep** | CLI | ⚠️ | ❌ | High | 8/10 |
| **ast-grep** | CLI | ⚠️ | ❌ | High | 8/10 |
| **comby** | CLI | ⚠️ | ❌ | High | 7/10 |

- **semgrep**: Best for large-scale rule-based migrations (e.g., API changes).
- **comby**: Fastest for language-agnostic structural search/replace.

---

## Final Synthesis for MCP Refactoring Server

For a Rust-based MCP Refactoring Server, the ideal architecture would be:

1. **TypeScript**: Wrap `ts-morph` in a lightweight Node script invoked by Rust.
2. **Python**: Use `ropecli` for deep refactors or `pyrefly` for performance.
3. **Rust**: Use `rust-analyzer` in headless mode via JSON-RPC.
4. **General Patterns**: Use `ast-grep` or `semgrep` for simpler, pattern-based changes.
