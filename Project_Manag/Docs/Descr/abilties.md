# Capabilities & Supported Languages

The `refac_mcp` server provides high-performance, intelligent refactoring capabilities directly within your Agentic workflow. It specializes in moving and renaming files while ensuring that references (imports/exports) are updated across the entire project.

## 1. Core Utilities

* **Intelligent Move/Rename**: Moves files or directories and updates all internal pointers using the appropriate language server for the task.
* **Batch Operations**: Execute multiple refactoring tasks for different files across different languages in a single MCP call.
* **Cross-Language Orchestration**: Seamlessly switches between different language drivers (LSP-based and custom) within a single batch.
* **Safety First**: Leverages industry-standard language servers to ensure that code logic remains intact after file moves.
* **Developer-Friendly Feedback**: Provides clear, code-comment styled (`//`) success messages that summarize all changes made.

## 2. Currently Supported Languages

The server integrates with the following language toolchains:

| Language | Driver Engine | Required Tooling |
| :--- | :--- | :--- |
| **Python** | `Rope` / `Pyrefly` | `python3`, `rope`, `pyrefly` |
| **TypeScript / JS** | `Bun` | `bun` |
| **Rust** | `rust-analyzer` | `rust-analyzer` binary |
| **Go** | `gopls` | `gopls` (Go Language Server) |
| **Dart** | `Dart SDK` | `dart language-server` |

## 3. Why Use `refac_mcp`?

Traditional file system moves in an Agentic environment often break imports because agents may forget to update every reference file. `refac_mcp` automates this maintenance work, ensuring that your project remains buildable and consistent even after large-scale structural changes.
