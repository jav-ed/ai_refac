# Project Findings & Decisions

## 1. SDK Selection

- **Decision**: Use the official [Rust SDK](https://github.com/modelcontextprotocol/rust-sdk) for the MCP implementation.
- **Action**: Cloned into `Repo/rust-sdk` for direct reference.

## 2. Architecture

- **Goal**: Create a unified MCP server (`refac_mcp`) that acts as a dispatcher.
- **Mechanism**:
  - Detect file extension of the target file.
  - Invoke the appropriate language-specific tool.
- **Target Languages & Selected Tools**:
    1. **TypeScript/JavaScript**: `ts-morph` (Node.js wrapper required).
    2. **Python**: `rope` (Python script wrapper required).
    3. **Rust**: `rust-analyzer` (LSP interaction required).

## 3. Open Questions

- **Integration Detail**: How to structure the "Sidecar" processes (Node/Python) within the Rust binary?
- **Packaging**: How to bundle these dependencies? or do we expect the user to have `node`/`python` installed? (Decision: Assume environment has them for now).
