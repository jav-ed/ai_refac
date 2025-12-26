# Refactoring Tool Research Findings (Expanded)

This report summarizes the deep research into the best backend tools for a Rust-based MCP Refactoring Server.

## Comprehensive Research Results

I have evaluated over 15 tools across multiple languages, focusing on their capability for deep refactoring (renaming symbols, moving files) and their potential for integration into a Rust-based server.

### 1. TypeScript / JavaScript

- **ts-morph (10/10)**: The definitive choice for project-wide semantic updates.
- **oxc (9/10)**: Best for high-performance transformations and linting fixes.
- **ast-grep (8/10)**: Excellent for structural pattern matching.

### 2. Python

- **rope (9/10)**: The gold standard for safe, deep refactors.
- **Pyrefly (9/10)**: Blazing fast type-checker/LSP from Meta with automatic import refactoring.
- **libcst (8/10)**: Best for building custom, lossless codemods.

### 3. Rust

- **rust-analyzer (9/10)**: The native solution for semantic refactoring via LSP.
- **IntelliJ Rust (9/10)**: Robust alternative engine with deep refactoring "assists".

### 4. Multi-Language

- **semgrep (8/10)**: Powerful for rule-based structural search and replace.
- **comby (7/10)**: Flexible and fast for syntax-aware pattern matching.

## Summary Recommendation

For the MCP Refactoring Server:

- **TypeScript**: Use **ts-morph** as the primary engine.
- **Python**: Use **rope** for deep refactors and Pyrefly for performance-critical tasks.
- **Rust**: Integrate with **rust-analyzer**'s LSP interface.
- **General**: Leverage **ast-grep** or **semgrep** for rule-based structural transformations.
