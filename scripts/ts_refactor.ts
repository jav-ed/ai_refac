import * as path from "node:path";
import { applyPlan } from "./TypeScript/apply";
import { prepareMoves } from "./TypeScript/moves";
import { planMoves, verifyPlan } from "./TypeScript/plan";
import { readProject } from "./TypeScript/project";
import { componentUsers } from "./TypeScript/users";
import { afterMoves } from "./TypeScript/virtual";

try {
    // `--dry-run` (anywhere after the command) prints the plan as JSON and writes nothing.
    // `--external <json_pairs>` names moves that another tool carries out afterwards
    // (pages and pictures the Markdown driver moves): the plan fixes the code that
    // uses them, and nothing renames them.
    const argv = process.argv.slice(2);
    const dryRun = argv.includes("--dry-run");
    const externalAt = argv.indexOf("--external");
    if (externalAt >= 0 && argv[externalAt + 1] === undefined) throw new Error("--external needs the JSON pairs of the moves another tool carries out.");
    const external: unknown = externalAt >= 0 ? JSON.parse(argv[externalAt + 1]!) : [];
    const flagged = (index: number) => externalAt >= 0 && (index === externalAt || index === externalAt + 1);
    const [command, ...args] = argv.filter((argument, index) => argument !== "--dry-run" && !flagged(index));
    if (command === "users" && args.length === 4) {
        // The component files that a rename of `name` would leave behind: of the given
        // extensions, they import from one of the JSON list of files that rename edits
        // and mention the name.
        const project = readProject(path.resolve(args[0]!));
        console.log(JSON.stringify(componentUsers(project, args[1]!, JSON.parse(args[2]!), JSON.parse(args[3]!))));
        process.exit(0);
    }
    let input: unknown;
    let root: string;
    if (command === "batch" && (args.length === 1 || args.length === 2)) {
        input = JSON.parse(args[0]!);
        root = path.resolve(args[1] ?? process.cwd());
    } else if (command === "move" && (args.length === 2 || args.length === 3)) {
        input = [[args[0], args[1]]];
        root = path.resolve(args[2] ?? process.cwd());
    } else {
        throw new Error("Usage: bun ts_refactor.ts batch <json_pairs> [project_root] [--external <json_pairs>], or move <source> <target> [project_root], or users <project_root> <name> <json_files> <json_extensions>");
    }
    const moves = prepareMoves(root, input, external);
    const project = readProject(root);
    process.stderr.write(`[refac] Scanning ${project.files.length} configured files with Oxc...\n`);
    const plan = planMoves(project, moves);
    // The last line is the plan as JSON; `moves` are the ones this engine carries out.
    const summary = (unchecked: number) => JSON.stringify({
        moves: moves.moves.filter(move => !move.external).map(move => ({ from: move.source, to: move.target })),
        files: plan.changes.filter(change => change.edits > 0).map(change => ({ path: change.file, edits: change.edits })),
        unchecked,
    });
    if (dryRun) {
        // The same check a real move makes once the files are in their new
        // places, against the files as they will be after the moves.
        console.log(summary(verifyPlan(project, plan, afterMoves(moves))));
    } else {
        const unchecked = applyPlan(project, plan);
        console.log(`Moved ${moves.moves.filter(move => !move.external).length} requested paths; verified ${plan.changes.length} affected source files.`);
        console.log(summary(unchecked));
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
