import * as fs from "node:fs";
import * as path from "node:path";
import { type Move } from "./moves";
import { type Plan, verifyPlan } from "./plan";
import { type ProjectConfig } from "./project";

export function applyPlan(project: ProjectConfig, plan: Plan): void {
    // Refuse concurrent source edits before the first write. The caller owns
    // exclusive access during this short apply/verify phase.
    for (const change of plan.changes) {
        if (fs.readFileSync(change.file, "utf8") !== change.before) {
            throw new Error(`Source changed while planning; refusing to overwrite ${change.file}`);
        }
    }
    for (const move of plan.moves.moves) {
        if (!fs.existsSync(move.source) || fs.existsSync(move.target)) {
            throw new Error(`Move paths changed while planning: ${move.source} -> ${move.target}`);
        }
    }
    const moved: Move[] = [];
    const written: typeof plan.changes = [];
    const createdDirectories: string[] = [];
    function mkdir(directory: string) {
        if (fs.existsSync(directory)) return;
        mkdir(path.dirname(directory));
        fs.mkdirSync(directory);
        createdDirectories.push(directory);
    }
    try {
        for (const change of plan.changes) {
            if (change.before === change.after) continue;
            written.push(change);
            fs.writeFileSync(change.file, change.after);
        }
        for (const move of plan.moves.moves) {
            mkdir(path.dirname(move.target));
            fs.renameSync(move.source, move.target);
            moved.push(move);
        }
        verifyPlan(project, plan);
    } catch (error) {
        const rollbackErrors: unknown[] = [];
        for (const move of moved.reverse()) {
            try { fs.renameSync(move.target, move.source); } catch (failure) { rollbackErrors.push(failure); }
        }
        for (const change of written) {
            try { fs.writeFileSync(change.file, change.before); } catch (failure) { rollbackErrors.push(failure); }
        }
        for (const directory of createdDirectories.reverse()) {
            try { fs.rmdirSync(directory); } catch (failure) { rollbackErrors.push(failure); }
        }
        if (rollbackErrors.length) throw new AggregateError([error, ...rollbackErrors], "Move failed and rollback was incomplete; inspect the working tree.");
        throw new Error(`Move failed; original files restored: ${error instanceof Error ? error.message : error}`, { cause: error });
    }
}
