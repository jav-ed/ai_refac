# MCP Refactor Server - Tasks

## Investigation

- [ ] Explore `Repo/rust-sdk` examples to understand server implementation.
- [ ] Determine the best "refactor drivers" for each language:
  - [ ] **Python**: (Candidates: `bowler`, `rope`, `libcst`?)
  - [ ] **TypeScript**: (Candidates: `ts-morph`, `jscodeshift`?)
  - [ ] **Rust**: (Candidates: `cargo fix`, `rust-analyzer` LSP?)

## Implementation

- [ ] Integrate `rust-sdk` into `Cargo.toml`.
- [ ] Create the main server loop.
- [ ] Implement the `refactor_file` tool definition.
- [ ] Implement the dispatcher logic (Extension -> Tool).
