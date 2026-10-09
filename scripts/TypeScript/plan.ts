import * as fs from "node:fs";
import * as path from "node:path";
import type ts from "typescript";
import { collectImports, quotePath, type ResolutionMode } from "./imports";
import { type MoveSet } from "./moves";
import { isSource, type ProjectConfig } from "./project";
import { makeResolver, replacement } from "./resolver";

export interface Check {
    specifier: string;
    expected: string;
    mode: ResolutionMode;
    /** True when the plan rewrites the import; an untouched one still reaches the file it reached before. */
    rewritten: boolean;
}
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
            checks.push({ specifier, expected, mode: reference.mode, rewritten: specifier !== reference.value });
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

/**
 * Resolves every rewritten specifier from the file's new place; it must reach
 * the file it reached before. `host` is the disk after a real move, or the
 * files as they will be after the moves for a dry run. Returns how many
 * rewritten specifiers could not be checked with that host. An import the plan
 * leaves alone cannot start answering differently, so it is not counted.
 */
export function verifyPlan(project: ProjectConfig, plan: Plan, host?: ts.ModuleResolutionHost): number {
    const resolver = makeResolver(project, host);
    let unchecked = 0;
    for (const change of plan.changes) {
        for (const check of change.checks) {
            if (!resolver.checkable(check.specifier)) {
                if (check.rewritten) unchecked++;
                continue;
            }
            const actual = resolver.resolve(change.target, check.specifier, check.mode);
            if (!actual || path.resolve(actual) !== check.expected) {
                throw new Error(`Moved module resolves incorrectly: ${change.target}: ${JSON.stringify(check.specifier)}; expected ${check.expected}, received ${actual ?? "unresolved"}. After the move another file or folder would answer that import, so nothing was changed; choose a target name that no other module answers.`);
            }
        }
    }
    return unchecked;
}
