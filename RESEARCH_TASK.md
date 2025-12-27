# Research Task: Dynamic Restart & Hot-Reloading for MCP Servers

## Context & Pain Point

Developing the `refac_mcp` server currently involves a painful iteration loop:

1. Modify Rust code.
2. Rebuild binary (`cargo build --release`).
3. Manually kill existing server processes (`pkill refac_mcp`).
4. Restart the client (Antigravity/Claude Code) or wait for it to crash and restart the server (which often fails producing EOF errors vs successful reconnects).

**Goal:** We need a workflow that allows us to modify the server code and have the changes take effect immediately in the active client *without* restarting the heavy client application.

## Research Objectives

Find a solution that enables:

1. **Hot-Reloading**: Automatically restarting the server binary when files change (e.g. `cargo watch`).
2. **Seamless Client Reconnection**: The MCP client should gracefully handle the server restart without crashing or requiring a full session reset.
3. **Local Debugging Harness**: A robust way to test the server loop locally that mimics the real agent environment better than our current `test_headless.ts`.

## Specific Questions to Answer

1. **MCP Protocol Support**: Does the Model Context Protocol have a native `notifications/restart` or `handshake/refresh` mechanism that tells the client to reconnect?
2. **Middleware/Proxy Solutions**: Is there an existing "MCP Proxy" that sits between the Client and the real Server?
    * The Proxy stays alive forever.
    * The Proxy manages the child Server process and kills/restarts it on file changes.
3. **Client-Specific Configuration**: Are there settings in Claude Code or Antigravity to enable "Auto-Restart on Crash" or "Watch Mode" for stdio servers?
4. **Existing Tools**: Look for tools like `mcp-inspector` but with auto-reload capabilities.

## Deliverable Format

Please provide the results in a Markdown document `research_results_reloading.md` with the following structure:

### 1. Recommended Solution

The single best approach to implement immediately.

### 2. Alternative Options

| Option | Mechanism | Pros | Cons | Est. Setup Time |
|--------|-----------|------|------|-----------------|
| e.g. Proxy | Node script wrapping binary | No client changes needed | Complexity | Medium |

### 3. Implementation Sketch

A brief code snippet or logic flow for the Recommended Solution (e.g., how to write the Node.js proxy wrapper).
