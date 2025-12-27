
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";

// Configuration
const SERVER_DIR = process.cwd();
const SERVER_CMD = "cargo";
const SERVER_ARGS = ["run", "--bin", "refac_mcp", "--quiet"];

// Dart Multi-File Test Configuration
const TRIALS_ROOT = join(SERVER_DIR, "Trials", "0_Refac_Tree", "dart");
const TEST_BATCH = [
    { src: "lib/models/app_model.dart", dst: "lib/core/app_model.dart" },
    { src: "lib/services/api_service.dart", dst: "lib/core/api_service.dart" }
];

async function runTest() {
    console.log("=== Agent Simulation: Real Refactor in Trials/ ===");
    console.log(`Server Dir:  ${SERVER_DIR}`);
    console.log(`Target Project: ${TRIALS_ROOT}`);

    for (const { src } of TEST_BATCH) {
        const srcAbs = join(TRIALS_ROOT, src);
        if (!existsSync(srcAbs)) {
            console.error(`❌ Setup Error: Source file ${srcAbs} missing.`);
            process.exit(1);
        }
    }

    console.log("Spawning server...");
    const proc = spawn(SERVER_CMD, SERVER_ARGS, {
        cwd: SERVER_DIR,
        stdio: ["pipe", "pipe", "inherit"],
        env: process.env
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

            if (msg.id === 1 && msg.result) {
                console.log("✅ Handshake complete.");
                send({ jsonrpc: "2.0", method: "notifications/initialized" });

                console.log("🚀 Sending multi-file 'refactor.move' request...");
                send({
                    jsonrpc: "2.0",
                    id: 2,
                    method: "tools/call",
                    params: {
                        name: "refactor",
                        arguments: {
                            operation: "move",
                            project_path: TRIALS_ROOT,
                            source_path: TEST_BATCH.map(b => b.src),
                            target_path: TEST_BATCH.map(b => b.dst)
                        }
                    }
                });
            }

            else if (msg.method === "roots/list") {
                send({
                    jsonrpc: "2.0",
                    id: msg.id,
                    result: {
                        roots: [{ uri: `file://${TRIALS_ROOT}`, name: "Trials" }]
                    }
                });
            }

            else if (msg.id === 2) {
                console.log("📩 Received Result.");

                if (msg.result && !msg.result.isError) {
                    console.log("✅ MCP Success.");

                    let allPassed = true;
                    for (const { src, dst } of TEST_BATCH) {
                        const srcAbs = join(TRIALS_ROOT, src);
                        const dstAbs = join(TRIALS_ROOT, dst);
                        if (existsSync(dstAbs) && !existsSync(srcAbs)) {
                            console.log(`✅ VERIFIED: ${src} -> ${dst}`);
                        } else {
                            console.error(`❌ FAILED: ${src} -> ${dst}`);
                            allPassed = false;
                        }
                    }

                    if (allPassed) {
                        console.log("🎉 SIMULATION PASSED!");
                        proc.kill();
                        process.exit(0);
                    } else {
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
