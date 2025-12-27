# Implementation Plan - Modular Refactoring Server

## Goal Description

Refactor the initial monolithic `main.rs` into a modular architecture to ensure specific file limits (<300 lines) and separation of concerns. Implement strict sanity checks and language drivers.

## Proposed Changes

### Structure

- `src/main.rs`: Minimal entry point.
- `src/server/mod.rs`: `RefactorServer` implementation.
- `src/validation/mod.rs`: Input validation and sanity checks.
- `src/logic/mod.rs`: Core refactoring logic (dispatcher).
- `src/drivers/mod.rs`: Traits and implementations for language tools (`ts`, `python`, `rust`).
  - `src/drivers/python.rs`: Dispatcher for Python.
  - `src/drivers/python_rope.rs`: Rope-based implementation (Fallback).
  - [x] `src/drivers/python_pyrefly.rs`: Pyrefly-based implementation (Primary, via LSP).
  - [x] `src/drivers/typescript.rs`
  - [x] `src/drivers/lsp_client.rs`: Generic LSP client for code reuse.
  - [x] `src/drivers/rust.rs`
  - [x] `src/drivers/go.rs`: Go implementation via generic LspClient (gopls).
  - [x] `src/drivers/dart.rs`: Dart implementation via generic LspClient (Analysis Server).

### validation/mod.rs

- **Function**: `initial_sanity_check(source: &Vec<String>, op: &str)`
- **Checks**:
  - Files exist.
  - Path traversal attempts.
  - Operation supported.

## Verification Plan

- **Unit Tests**:
  - `cargo test validation`
- **Integration**:
  - Create a temporary dummy file.
  - Send a `move` request via MCP Stdio.
  - Verify dispatcher calls the mock driver.

## Phase 2: Batch Operations & Routing (New)

### Goal

Support multiple file moves in a single request, intelligently routing files to the correct driver.

### Changes

1. [x] **RefactorDriver Trait**: Change `move_file(src, tgt)` to `move_files(file_map: Vec<(String, String)>)`.
2. [x] **LspClient**: Update to accept `Vec<(String, String)>` and send generic `RenameFilesParams`.
3. [x] **Core Logic (`src/logic/mod.rs`)**:
    - [x] Group input files by language (extension).
    - [x] Instantiate driver *once* per language group.
    - [x] specialized `handle_refactor` to dispatch batch requests.
4. [x] **Driver Updates**: Update all implementation to handle the vector signature.

### Phase 2b: Script Batch Optimization (TypeScript & Rope)

To avoid O(N) initialization overhead for `ts-morph` and `rope`:

1. [x] **Refactor `scripts/ts_refactor.js`**: Add `batch` command taking a JSON string of moves. Initialize `Project` once, apply all moves, save once.
2. [x] **Refactor `scripts/python_refactor.py`**: Add `batch` command. Initialize `Project` once, perform sequential moves.
3. [x] **Update Drivers**: Update `TypeScriptDriver` and `RopeDriver` to construct the JSON payload and call the script once.
