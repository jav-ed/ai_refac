# Walkthrough - Modular Refactoring Server Implementation

## 1. Goal

To implement a modular, clean-architecture MCP server capable of refactoring (moving) files in multiple languages (Python, TypeScript, Rust) using standard industry tools via a unified interface.

## 2. Architecture

The system uses a dispatcher pattern (`RefactorServer` -> `Validation` -> `Logic` -> `Drivers`).

### Drivers

- **Python**: Composite driver.
  - **Primary**: `PyreflyDriver` (Rust-based). Integrated via a generic `LspClient` interacting with the `pyrefly` LSP binary.
  - **Fallback**: `RopeDriver` (Python-based). Interacts with a `python_refactor.py` script using the `rope` library.
- **TypeScript**: `TypeScriptDriver`. Interacts with `scripts/ts_refactor.ts` using `bun` (replacing Node.js) and `ts-morph` for robust AST manipulations and directory moves.
- **Rust**: `RustDriver`. Integrated via the shared `LspClient` interacting with `rust-analyzer`.
- **Go**: `GoDriver`. Integrated via the shared `LspClient` interacting with `gopls`.
- **Dart**: `DartDriver`. Integrated via the shared `LspClient` interacting with `dart language-server`.

### 2.1 Batch Operations

Phase 2 implemented intelligent batch processing:

- **Grouping**: Input files are grouped by language (extension) in `src/logic/mod.rs`.
- **Atomic Dispatch**: A single request is sent to the driver for all files in a language group.
- **Optimization**:
  - **LSP Drivers** (Rust, Go, Dart, Pyrefly): Send a single `workspace/willRenameFiles` request.
  - **Script Drivers** (TypeScript, Rope): Use a new `batch` command with JSON payload. This allows initialization of the analysis engine (`ts-morph` project, `rope` project) **once per batch** (O(1)) instead of once per file (O(N)), significantly improving performance.

## 3. Verification

### Automated Tests

Ran `cargo test` which covers:

- **Validation Logic**: Path traversal prevention, file existence checks.
- **Driver Availability**: Checks if external tools (`pyrefly`, `python`, `node`, `rust-analyzer`) are present.
- **Integration**:
  - `test_ts_move_e2e`: Verifies `ts-morph` script execution (updated to use batch mode).
  - `test_python_dispatcher_availability`: Verifies Python driver composite logic.
  - `test_rust_availability`: Verifies `rust-analyzer` connection.
  - `test_gopls_availability`: Verifies `gopls` connection.
  - `test_dart_availability`: Verifies `dart` connection.

### Manual Verification

- **Pyrefly**: Confirmed LSP handshake and `workspace/willRenameFiles` request sending.
- **Clean Architecture**: Refactored LSP logic into `src/drivers/lsp_client.rs` to share code between Python and Rust drivers.
- **Batch Optimization**: Verified that `scripts/ts_refactor.js` and `scripts/python_refactor.py` correctly parse JSON payloads and process multiple files in a single run.

## 4. Usage

The server expose an implementation prompt that uses these drivers. The core interface `handle_refactor` now accepts a list of sources and targets, automatically efficiently routing them.

## 5. Recent Enhancements (User Feedback Integration)

### 5.1 Project Context & Path Resolution

- **Issue**: Relative paths were resolving incorrectly (relative to server binary instead of user project) or failing entirely.
- **Fix**: Added optional `project_path` parameter to the API. Updated all drivers to pass this path to underlying tools. `ts-morph` and other scripts now resolve all paths physically against this root.
- **Regression Testing**: Added `test_resolve_resource_path_from_foreign_dir` to ensure the server finds its own scripts matching the binary location, regardless of CWD.

### 5.2 TypeScript Directory Support

- **Insight**: Analysis of previous `typescript_refactor_bridge.ts` revealed native support for directory moves (moving a folder and automatically updating imports in all children).
- **Implementation**: Updated `scripts/ts_refactor.js` to detect if a source is a directory. If so, it uses `project.getDirectory(src).move(target)` instead of file-only logic. This allows moving entire components (`src/auth`) with one operation.
