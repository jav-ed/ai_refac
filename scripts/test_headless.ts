
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";

// Configuration
const SERVER_DIR = process.cwd();
const SERVER_CMD = "cargo";
const SERVER_ARGS = ["run", "--bin", "refac_mcp", "--quiet"];

// Point to the REAL generated testbed
const TRIALS_ROOT = join(SERVER_DIR, "Trials", "0_Refac_Tree", "typescript");
const TEST_SRC = "src/models/TaskManager.ts";
const TEST_DST = "src/core/TaskManager.ts";

const SRC_ABS = join(TRIALS_ROOT, TEST_SRC);
const DST_ABS = join(TRIALS_ROOT, TEST_DST);

async function runTest() {
    console.log("=== Agent Simulation: Real Refactor in Trials/ ===");
    console.log(`Server Dir:  ${SERVER_DIR}`);
    console.log(`Target Project: ${TRIALS_ROOT}`);

    if (!existsSync(SRC_ABS)) {
        console.error(`❌ Setup Error: Source file ${SRC_ABS} missing. Did you run 'cargo run --bin create_testbed'?`);
        process.exit(1);
    }

    console.log("Spawning server...");
    const proc = spawn(SERVER_CMD, SERVER_ARGS, {
        cwd: SERVER_DIR,
        stdio: ["pipe", "pipe", "inherit"],
        env: process.env // Inherit PATH so bun is found
    });

    let buffer = "";

    proc.stdout.on("data", (chunk) => {
        buffer += chunk.toString();

        let newlineIdx;
        while ((newlineIdx = buffer.indexOf("\n")) !== -1) {
            const line = buffer.slice(0, newlineIdx);
            buffer = buffer.slice(newlineIdx + 1);

            if (!line.trim()) continue;
            handleMessage(line);
        }
    });

    function send(msg: any) {
        const str = JSON.stringify(msg) + "\n";
        proc.stdin.write(str);
    }

    function handleMessage(line: string) {
        try {
            const msg = JSON.parse(line);

            // 1. Handshake
            if (msg.id === 1 && msg.result) {
                console.log("✅ Handshake complete.");
                send({ jsonrpc: "2.0", method: "notifications/initialized" });

                console.log("🚀 Sending 'refactor.move' request...");
                send({
                    jsonrpc: "2.0",
                    id: 2,
                    method: "tools/call",
                    params: {
                        name: "refactor",
                        arguments: {
                            operation: "move",
                            // Explicitly providing project_path for this simulation
                            // This mimics the 'Fix 1' we suggested, but auto-detection would work too
                            project_path: TRIALS_ROOT,
                            source_path: [TEST_SRC],
                            target_path: [TEST_DST]
                        }
                    }
                });
            }

            // 2. Roots (Auto-detection fallback, though we provided path)
            else if (msg.method === "roots/list") {
                console.log("👀 Server asked for roots.");
                send({
                    jsonrpc: "2.0",
                    id: msg.id,
                    result: {
                        roots: [{ uri: `file://${TRIALS_ROOT}`, name: "Trials" }]
                    }
                });
            }

            // 3. Result
            else if (msg.id === 2) {
                console.log("📩 Received Result.");

                if (msg.result && !msg.result.isError) {
                    console.log("✅ MCP Success.");

                    // Verify file system
                    if (existsSync(DST_ABS) && !existsSync(SRC_ABS)) {
                        console.log("✅ VERIFIED: File moved on disk.");
                        console.log("🎉 SIMULATION PASSED!");
                        proc.kill();
                        process.exit(0);
                    } else {
                        console.error("❌ File Verification FAILED.");
                        proc.kill();
                        process.exit(1);
                    }
                } else {
                    console.error("❌ MCP Error:", JSON.stringify(msg, null, 2));
                    proc.kill();
                    process.exit(1);
                }
            }
        } catch (e) { }
    }

    // 0. Start
    send({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
            protocolVersion: "2024-11-05",
            capabilities: { roots: { listChanged: true } },
            clientInfo: { name: "agent-simulator", version: "1.0.0" },
        },
    });
}

runTest();
