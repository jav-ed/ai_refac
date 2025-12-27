# Research Results: Dynamic Restart & Hot-Reloading for MCP Servers

## 1. Recommended Solution: `mcpmon` (Transparent HMR Proxy)

The single best approach to implement immediately is using **`mcpmon`** as a wrapper for your `refac_mcp` server.

### Why `mcpmon`?

- **Zero Client Changes**: It acts as a transparent proxy. The client (Antigravity/Claude Code) connects to `mcpmon`, which then spawns and manages the `refac_mcp` process.
- **Request Buffering**: Crucially, it buffers incoming JSON-RPC requests while the server is restarting. This prevents the "EOF" or "Connection Lost" errors that usually crash the client session.
- **Auto-Watch**: It monitors file changes (like `nodemon`) and triggers a rebuild/restart automatically.
- **Cross-Language**: Works perfectly with Rust, Node.js, and Python servers.

---

## 2. Alternative Options

| Option | Mechanism | Pros | Cons | Est. Setup Time |
|--------|-----------|------|------|-----------------|
| **mcpmon** (Recommended) | Node.js HMR Proxy wrapping binary | **Buffering requests during reload**, zero client config, battle-tested for MCP. | Requires Node.js installed locally. | 5 mins |
| **reloaderoo** | TS-based Proxy + Tool UI | Provides an **agent-triggered restart tool**, powerful inspection UI. | Slightly heavier than mcpmon. | 10 mins |
| **mcp-hot-reload** | Python wrapper | Simple, enables Claude to restart server via tool. | Python dependency; less robust request buffering than mcpmon. | 10 mins |
| **Native MCP `notifications/tools/list_changed`** | Protocol-level notify | Officially supported way to refresh tool definitions. | Doesn't handle the binary process restart/hot-swap; requires client support. | High (Needs server-side implementation) |

---

## 3. Implementation Sketch (using `mcpmon`)

To implement this workflow for `refac_mcp`:

1. **Install `mcpmon`**:

   ```bash
   npm install -g mcpmon
   ```

2. **Run with Watch Mode**:
   Configure your MCP client to use the following command instead of the direct path to your binary:

   ```bash
   mcpmon --watch ./src --cmd "cargo run --release"
   ```

### Logic Flow for custom Proxy (if building from scratch)

If you prefer a custom Node.js script to manage the lifecycle:

```javascript
const { spawn } = require('child_process');
const fs = require('fs');

let child = null;
const spawnServer = () => {
    if (child) child.kill();
    // Rebuild and start
    child = spawn('cargo', ['run', '--release'], { stdio: ['pipe', 'pipe', 'inherit'] });
    
    // Pipe stdin from proxy to child, and stdout from child to proxy
    process.stdin.pipe(child.stdin);
    child.stdout.pipe(process.stdout);
    
    child.on('exit', () => console.error('Server exited, waiting for changes...'));
};

// Simple watch trigger
fs.watch('./src', { recursive: true }, (event, filename) => {
    console.error(`File ${filename} changed, restarting...`);
    spawnServer();
});

spawnServer();
```

---

## 4. Specific Answers to Research Questions

1. **MCP Protocol Support**: The protocol does **not** have a native "force client to reconnect" notification. However, it has `notifications/resources/updated` and `notifications/tools/list_changed` which tell the client to refresh its internal models of the server's capabilities without a full restart.
2. **Middleware/Proxy Solutions**: Yes, `mcpmon` and `reloaderoo` are the industry-leading proxies for this exact pain point. They stay alive while their children die and are reborn.
3. **Client-Specific Configuration**: Claude Code and Antigravity do not currently have a "watch" mode for *stdio* servers. They expect the process to be stable. The proxy approach is the standard workaround.
4. **Existing Tools**: `mcp-inspector` is great for debugging individual calls, but `reloaderoo` is the current state-of-the-art for "Inspector + Proxy + Hot Reload".
