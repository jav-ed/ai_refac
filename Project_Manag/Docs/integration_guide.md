# Integration Guide: Adding Refac MCP to AI Assistants

This guide explains how to configure various AI coding assistants to use the `refac_mcp` server.

## Prerequisites

First, ensure you have built the release binary of the server:

```bash
cargo build --release
# The binary will be located at:
# <project_root>/target/release/refac_mcp
```

Get the absolute path to this binary. For the examples below, we will use `/absolute/path/to/refac_mcp`.

---

## 1. Claude Code / Claude Desktop

Claude uses a JSON configuration file to manage MCP servers.

### CLI Method (Claude Code)

If you are using the `claude` CLI tool:

```bash
claude mcp add refactor_server --transport stdio -- /absolute/path/to/refac_mcp
```

### Manual Config Method (Claude Desktop)

Edit your config file typically located at:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Add the following entry to the `mcpServers` object:

```json
{
  "mcpServers": {
    "refactor_server": {
      "command": "/absolute/path/to/refac_mcp",
      "args": [],
      "env": {
        "PATH": "/usr/local/bin:/usr/bin:/bin" 
      }
    }
  }
}
```

*Note: Ensure the `PATH` environment variable includes locations of tools like `node`, `python`, `go`, and `dart` so the server can find them.*

---

## 2. Google Antigravity (Agentic IDE)

To add this server to a Google Antigravity agent session:

1. Open your **Agent Session**.
2. Click the **"..." (More options)** menu in the side panel.
3. Select **MCP Servers** -> **Manage MCP Servers**.
4. Switch to the **"Raw Config"** or **"JSON"** tab.
5. Add your server to the configuration array/object:

```json
{
  "refactor_server": {
    "command": "/absolute/path/to/refac_mcp",
    "transport": "stdio"
  }
}
```

*Note: If Antigravity runs in a container, you may need to ensure the binary is accessible or compile it for the specific container environment.*

---

## 3. OpenAI Codex (CLI / IDE Extension)

For the OpenAI Codex CLI or compatible extensions, configurations are typically stored in `~/.codex/config.toml`.

### CLI Method

```bash
codex mcp add refactor_server --command "/absolute/path/to/refac_mcp"
```

### Manual Config Method (`config.toml`)

```toml
[mcp_servers.refactor_server]
command = "/absolute/path/to/refac_mcp"
args = []
```

---

## 4. VS Code (via MCP Extension)

If you are using an MCP extension for VS Code:

1. Create or edit `.vscode/mcp.json` in your project root.
2. Add the server configuration:

```json
{
  "mcpServers": {
    "refactor_server": {
      "command": "/absolute/path/to/refac_mcp",
      "args": []
    }
  }
}
```

3. Restart VS Code or reload the window to apply changes.
