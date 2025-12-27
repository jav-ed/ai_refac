# Developer & Build Guide

This document explains how to build the `refac_mcp` server and utilize the integrated testbed generator for verification.

## 1. Building the Project

The server is written in Rust and should be built using `cargo`. For production use within an MCP environment, the release binary is recommended for performance.

```bash
# Build the project in release mode
cargo build --release
```

The resulting binary will be located at `target/release/refac_mcp`.

## 2. The Testbed Generator

To verify refactoring operations across multiple languages without risking production code, use the `create_testbed` utility.

### Running the Generator

```bash
# Run the testbed generator
cargo run --bin create_testbed
```

### What it does

The `create_testbed` command performs the following actions:

1. **Cleans the target directory**: It wipes the `Trials/0_Refac_Tree` directory to ensure a fresh state.
2. **Generates 5 Projects**: It creates functional, multi-file skeletal projects for every supported language:
    * **TypeScript**: A Task Manager domain structure.
    * **Python**: An E-Commerce domain structure.
    * **Rust**: A Game Engine domain structure.
    * **Go**: A Banking domain structure.
    * **Dart**: An App Store domain structure.
3. **Cross-Project Complexity**: Each project includes internal dependencies (imports/mod declarations) to verify that the refactoring drivers correctly update reference pointers across files.

## 3. Verification Workflow

1. Build the server: `cargo build --release`.
2. Generate fresh test data: `cargo run --bin create_testbed`.
3. Restart your MCP session to load the new binary.
4. Perform refactoring operations on files within `Trials/0_Refac_Tree` and observe the results.
