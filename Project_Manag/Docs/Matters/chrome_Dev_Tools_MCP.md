# Chrome DevTools MCP setup (Claude Code + Codex CLI)

Use this to remember why the Chrome DevTools MCP needed extra config and how to keep it working.

## What was broken
- `chrome-devtools-mcp` refuses Node `< 22.12`. Our system default was Node `22.2.0`, so the server crashed with: ``does not support Node v22.2.0``.
- Claude Code inherits your shell (with `nvm` defaulting to Node 23), so it worked once restarted. Codex CLI does **not** inherit shell init, so it kept using Node 22 until we hard-set `PATH`.
- The MCP launches Chrome/Brave headful; Codex needed `DISPLAY` set to use your running X session.

## Current configs
- **Claude Code project config**: `.mcp.json`
  ```json
  {
    "chrome-devtools": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "-y",
        "chrome-devtools-mcp@latest",
        "--executable-path",
        "/home/jav/Progs/brave/brave-browser"
      ]
    }
  }
  ```
  - Relies on your shell `PATH` pointing to Node 23 (via `nvm`). Restart Claude Code after edits.

- **Codex CLI global config**: `~/.codex/config.toml`
  ```toml
  [mcp_servers.chrome_devtools]
  command = "npx"
  args = ["-y", "chrome-devtools-mcp@latest", "--executable-path", "/home/jav/Progs/brave/brave-browser"]
  env = { PATH = "/home/jav/.nvm/versions/node/v23.11.1/bin:/usr/local/bin:/usr/bin:/bin", DISPLAY = ":1" }
  startup_timeout_sec = 40.0
  ```
  - PATH forces Node 23 for this server.
  - DISPLAY points to the running X session so headful works.

## Reproduction / troubleshooting
- Error about Node version → install/update Node (e.g., `nvm install 23.11.1 && nvm alias default 23`) and ensure the config points at that Node (`PATH` override for Codex).
- Error about missing X server → ensure `DISPLAY` is set (Codex env above) and an X session is running. If truly headless, switch to `--chrome-arg=--headless=new` instead.
- Browser not found → update `--executable-path` to the actual Chrome/Brave binary.

## When changing nodes or browsers
- Update both configs if the Node install path or browser path changes.
- Restart the respective client (Claude Code or Codex) after edits.
