import * as fs from "node:fs";
import * as path from "node:path";
import { collectImports, quotePath, type ResolutionMode } from "./imports";
import { type MoveSet } from "./moves";
import { isSource, type ProjectConfig } from "./project";
import { makeResolver, replacement } from "./resolver";

export interface Check { specifier: string; expected: string; mode: ResolutionMode; }
export interface FileChange {
    file: string;
    target: string;
    before: string;
    after: string;
    /** How many import specifiers the plan rewrites in this file. */
    edits: number;
    checks: Check[];
}
export interface Plan { moves: MoveSet; changes: FileChange[]; }

export function planMoves(project: ProjectConfig, moves: MoveSet): Plan {
    const resolver = makeResolver(project);
    // Preserve complete caller coverage; moving fewer files never narrows the
    // project scan. Explicitly moved sources may sit outside tsconfig's include.
    const files = new Set([...project.files.filter(isSource), ...[...moves.files.keys()].filter(isSource)]);
    const changes: FileChange[] = [];
    for (const file of files) {
        const before = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(fs.readFileSync(file));
        const target = moves.files.get(file) ?? file;
        const checks: Check[] = [];
        const edits: { start: number; end: number; text: string }[] = [];
        for (const reference of collectImports(file, before)) {
            const resolved = resolver.resolve(file, reference.value, reference.mode);
            if (!resolved) {
                if (file !== target && reference.value.startsWith(".")) {
                    throw new Error(`Cannot move ${file}: relative module ${JSON.stringify(reference.value)} cannot be resolved. No files were changed.`);
                }
                continue;
            }
            const expected = moves.files.get(path.resolve(resolved)) ?? path.resolve(resolved);
            const affected = expected !== path.resolve(resolved) || (file !== target && reference.value.startsWith("."));
            const specifier = affected ? replacement(project, reference.value, target, expected, reference.mode) : reference.value;
            checks.push({ specifier, expected, mode: reference.mode });
            if (specifier !== reference.value) {
                edits.push({ start: reference.start, end: reference.end, text: quotePath(specifier, before[reference.start]!) });
            }
        }
        if (edits.length === 0 && file === target) continue;
        let after = before;
        // Offsets come from the original AST. Apply all replacements backwards
        // once, keeping every other byte (comments, spacing, quote style) intact.
        for (const edit of edits.sort((a, b) => b.start - a.start)) {
            after = after.slice(0, edit.start) + edit.text + after.slice(edit.end);
        }
        changes.push({ file, target, before, after, edits: edits.length, checks });
    }
    return { moves, changes };
}

export function verifyPlan(project: ProjectConfig, plan: Plan): void {
    const resolver = makeResolver(project);
    for (const change of plan.changes) {
        for (const check of change.checks) {
            const actual = resolver.resolve(change.target, check.specifier, check.mode);
            if (!actual || path.resolve(actual) !== check.expected) {
                throw new Error(`Moved module resolves incorrectly: ${change.target}: ${JSON.stringify(check.specifier)}; expected ${check.expected}, received ${actual ?? "unresolved"}`);
            }
        }
    }
}
