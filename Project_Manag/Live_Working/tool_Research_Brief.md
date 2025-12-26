# Refactoring Tool Research Brief

## Objective

Identify the best "backend" tools to power a Rust-based MCP Refactoring Server. The server will invoke these tools to perform atomic refactoring operations.

## Requirements

1. **Capabilities**:
    - **Variable/Symbol Renaming**: Must update all references across the codebase.
    - **File Move/Rename**: Must update all imports/requires that reference the file.
2. **Performance**: Must be fast enough for interactive use.
3. **Integration**:
    - Preference for CLI tools that can be invoked with JSON arguments.
    - Alternatively, libraries that can be easily wrapped in a lightweight script.
    - **Rust Integration**: The parent process is a Rust server.

## Target Languages & Candidates to Evaluate

### 1. TypeScript / JavaScript

- **Candidates**: `ts-morph`, `jscodeshift`, `typescript` (compiler API), `ast-grep`.
- **Key Question**: Which tool reliably handles `move_file` (import updates) with minimal setup? `ts-morph` is powerful but requires a Node process. Is there a faster Rust-native alternative (like `ast-grep` or `oxc`) that is mature enough for *writing* changes?

### 2. Python

- **Candidates**: `rope`, `bowler`, `libcst`, `jedi`.
- **Key Question**: `rope` is the gold standard for renaming, but can `bowler` or `libcst` handle it more robustly? Is there a CLI wrapper for `rope` that outputs machine-readable diffs?

### 3. Rust

- **Candidates**: `rust-analyzer` (LSP), `cargo fix`.
- **Key Question**: Can we invoke `rust-analyzer`'s rename/move capabilities strictly from the command line (headless) without a persistent LSP session, or do we need to manage an LSP client process?

## Output Format

Please provide the findings in the following JSON format for each language:

```json
{
  "language": "TypeScript",
  "tools": [
    {
      "name": "Tool Name",
      "type": "CLI | Library | LSP",
      "pros": ["..."],
      "cons": ["..."],
      "supports_rename": true,
      "supports_file_move": true,
      "recommendation_score": 1-10
    }
  ],
  "best_choice": "Name of the winner"
}
```
