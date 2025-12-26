# MCP Refactor Server - Tasks

## Investigation

- [x] Explore `Repo/rust-sdk` examples to understand server implementation.
- [x] Determine the best "refactor drivers" for each language:
  - [x] **Python**: Pyrefly (Primary) + Rope (Fallback)
  - [x] **TypeScript**: ts-morph
  - [x] **Rust**: rust-analyzer

## Implementation

- [x] Integrate `rust-sdk` into `Cargo.toml`.
- [x] Create the main server loop.
- [x] Implement the `refactor_file` tool definition.
- [x] Implement the dispatcher logic (Extension -> Tool).
