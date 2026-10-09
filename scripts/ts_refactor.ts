import * as path from "node:path";
import { applyPlan } from "./TypeScript/apply";
import { prepareMoves } from "./TypeScript/moves";
import { planMoves, verifyPlan } from "./TypeScript/plan";
import { readProject } from "./TypeScript/project";
import { afterMoves } from "./TypeScript/virtual";

try {
    // `--dry-run` (anywhere after the command) prints the plan as JSON and writes nothing.
    const dryRun = process.argv.includes("--dry-run");
    const [command, ...args] = process.argv.slice(2).filter(argument => argument !== "--dry-run");
    let input: unknown;
    let root: string;
    if (command === "batch" && (args.length === 1 || args.length === 2)) {
        input = JSON.parse(args[0]!);
        root = path.resolve(args[1] ?? process.cwd());
    } else if (command === "move" && (args.length === 2 || args.length === 3)) {
        input = [[args[0], args[1]]];
        root = path.resolve(args[2] ?? process.cwd());
    } else {
        throw new Error("Usage: bun ts_refactor.ts batch <json_pairs> [project_root], or move <source> <target> [project_root]");
    }
    const moves = prepareMoves(root, input);
    const project = readProject(root);
    process.stderr.write(`[refac] Scanning ${project.files.length} configured files with Oxc...\n`);
    const plan = planMoves(project, moves);
    if (dryRun) {
        // The same check a real move makes once the files are in their new
        // places, against the files as they will be after the moves.
        const unchecked = verifyPlan(project, plan, afterMoves(moves));
        console.log(JSON.stringify({
            moves: moves.moves.map(move => ({ from: move.source, to: move.target })),
            files: plan.changes.filter(change => change.edits > 0).map(change => ({ path: change.file, edits: change.edits })),
            unchecked,
        }));
    } else {
        applyPlan(project, plan);
        console.log(`Moved ${moves.moves.length} requested paths; verified ${plan.changes.length} affected source files.`);
    }
} catch (error) {
    // The message is the answer. Printing the error object would bury it under
    // Bun's code frames and stack traces.
    console.error(describe(error));
    process.exitCode = 1;
}

function describe(error: unknown): string {
    if (error instanceof AggregateError) return [error.message, ...error.errors.map(describe)].join("\n");
    return error instanceof Error ? error.message : String(error);
}
