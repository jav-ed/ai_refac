# Testing & Debugging Guide

This guide describes how to test the Refactor MCP Server headlessly and how to debug it using the MCP Inspector.

## 1. Headless Testing Script

We provide a robust TypeScript script to verify server functionality without needing a full UI client (like Claude Desktop or an IDE).

### Script Location

`scripts/test_headless.ts`

### Prerequisites

- [Bun](https://bun.sh) (v1.0+) installed.
- The server project built (`cargo build`).

### How to Run

Run the script from the project root:

```bash
bun scripts/test_headless.ts
```

### What it Does

1. **Spawns the Server**: Runs `cargo run` to start the MCP server.
2. **Mock Client**: Simulates an MCP client that supports the `roots` capability.
3. **Execution**:
    - Performs the MCP initialization handshake.
    - Creates a temporary dummy file (`test_src_headless.js`) for testing.
    - Sends a `refactor` request (intentionally omitting `project_path` to test auto-detection).
    - Listens for the `roots/list` request from the server and responds with the current directory.
    - Verifies the file was successfully moved to `test_dst_headless.js`.
4. **Cleanup**: Deletes the temporary test files.

## 2. Interactive Debugging with MCP Inspector

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is a web-based tool provided by Anthropic/ModelContextProtocol to interactively test MCP servers.

### How to Run

Use `bunx` to launch the inspector with your server:

```bash
bunx @modelcontextprotocol/inspector cargo run
```

### Features

- **Interactive UI**: View available tools and resources.
- **Interactive UI**: View available tools and resources.
- **Tool Testing**: Fill in arguments for `refactor` and execute them to see real-time logs and responses.
- **Log Inspection**: View `tracing` logs output by the Rust server in the "Notifications" or "Logs" panel.

### Debugging Tips

#### "Source path does not exist" Error

If you see this error, it means the server cannot resolve the relative path you provided.

- **Fix 1 (Explicit)**: Provide the `project_path` argument in your JSON payload.

    ```json
    {
      "operation": "move",
      "project_path": "/absolute/path/to/project",
      "source_path": ["src/file.ts"],
      "target_path": ["src/new_file.ts"]
    }
    ```

- **Fix 2 (Auto-Detection)**: Ensure your client (or the Inspector) supports the `roots` capability. The server will automatically ask for the workspace root. *Note: The standard web-based Inspector might not fully mock `roots` auto-detection yet depending on version, so explicit `project_path` is safer there.*

## 3. Persistent Testbed Generator

To verify complex refactoring scenarios, you can generate a consistent, multi-language test folder (`Trials/0_Refac_Tree`) with valid, interconnected code.

### How to Run

```bash
cargo run --bin create_testbed
```

### What it Does

1. **Wipes** `Trials/0_Refac_Tree` (if it exists).
2. **Creates** a structured playground with projects for:
    - **TypeScript**: `typescript/` (with `tsconfig.json`, `src/index.ts`, imports)
    - **Python**: `python/` (with `lib/math_utils.py`, `main.py`)
    - **Rust**: `rust/` (with `Cargo.toml`, `src/models/user.rs`)
    - **Go**: `go/` (with `go.mod`, `pkg/util.go`)

Use this directory as your `project_path` (e.g., `<ROOT>/Trials/0_Refac_Tree/typescript`) when testing with the Inspector or scripts to ensure you are working with valid project structures.
