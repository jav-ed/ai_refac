# Integration Guide: Adding Refac MCP to AI Assistants

This guide explains how to configure various AI coding assistants to use the `refac_mcp` server.

## 1. Path Configuration Rules

The most common error is pointing to the **project folder** instead of the **executable binary**.

The path is composed of two parts:

1. **`<PROJECT_ROOT>`**: The absolute path to where you cloned/downloaded this repository.
    * *Example*: `/home/username/code/refac_mcp`
2. **`<BINARY_PATH>`**: The constant location of the compiled executable inside the project.
    * *Value*: `/target/release/refac_mcp`

**Final Command to Use:**

```text
<PROJECT_ROOT>/target/release/refac_mcp
```

### Example Construction

If your project is at `/home/jav/code/mcp/refac_mcp`, your full command string is:
`/home/jav/code/mcp/refac_mcp/target/release/refac_mcp`

---

## 2. Global Integration Steps

First, ensure you have built the release binary:

```bash
cd <PROJECT_ROOT>
cargo build --release
```

---

## 1. Claude Code / Claude Desktop

Claude uses a JSON configuration file to manage MCP servers.

### CLI Method (Claude Code)

If you are using the `claude` CLI tool:

```bash
claude mcp add refactor_server --transport stdio -- <PROJECT_ROOT>/target/release/refac_mcp
```

### Manual Config Method (Claude Desktop)

Edit your config file typically located at:

* **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
* **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Add the following entry to the `mcpServers` object:

```json
{
  "mcpServers": {
    "refactor_server": {
      "command": "/your/absolute/path/to/projects/refac_mcp/target/release/refac_mcp",
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
    "command": "/your/absolute/path/to/projects/refac_mcp/target/release/refac_mcp",
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
codex mcp add refactor_server --command "<PROJECT_ROOT>/target/release/refac_mcp"
```

### Manual Config Method (`config.toml`)

```toml
[mcp_servers.refactor_server]
command = "/your/absolute/path/to/refac_mcp/target/release/refac_mcp"
args = []
```

---

## 6. VS Code (via MCP Extension)

If you are using an MCP extension for VS Code:

1. Create or edit `.vscode/mcp.json` in your project root.
2. Add the server configuration:

```json
{
  "mcpServers": {
    "refactor_server": {
      "command": "/your/absolute/path/to/refac_mcp/target/release/refac_mcp",
      "args": []
    }
  }
}
```

1. Restart VS Code or reload the window to apply changes.
