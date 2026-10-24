import * as fs from "node:fs";
import * as path from "node:path";
import ts from "typescript";
import { quoteAttribute, quoteCssPath, quotePath, scanFile, type ModuleReference, type ResolutionMode } from "./imports";
import { type MoveSet } from "./moves";
import { isSource, type ProjectConfig } from "./project";
import { makeResolver, replacement, splitSuffix } from "./resolver";
import { globEdits, pathStringEdits, relativePathEdits, verifyPathStrings } from "./strings";

export interface Check {
    specifier: string;
    expected: string;
    mode: ResolutionMode;
    /** True when the plan rewrites the import; an untouched one still reaches the file it reached before. */
    rewritten: boolean;
    /** The specifier is a path of a stylesheet (see `resolve`). */
    stylesheet: boolean;
}
export interface FileChange {
    file: string;
    target: string;
    before: string;
    after: string;
    /** How many import specifiers the plan rewrites in this file. */
    edits: number;
    checks: Check[];
    /** Paths written as strings that the plan rewrites; each must exist after the move. */
    paths: string[];
}
export interface Plan { moves: MoveSet; changes: FileChange[]; }

export function planMoves(project: ProjectConfig, moves: MoveSet): Plan {
    const resolver = makeResolver(project);
    // Preserve complete caller coverage; moving fewer files never narrows the
    // project scan. Explicitly moved sources may sit outside tsconfig's include.
    const files = new Set([...project.files.filter(isSource), ...[...moves.files.keys()].filter(isSource)]);
    const changes: FileChange[] = [];
    for (const file of files) {
        const bytes = fs.readFileSync(file);
        // A plain `.md` without frontmatter names nothing, and its encoding is not our business.
        if (file.endsWith(".md") && !hasFrontmatterFence(bytes)) continue;
        const before = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
        const target = moves.files.get(file) ?? file;
        const checks: Check[] = [];
        const edits: { start: number; end: number; text: string }[] = [];
        const { modules, strings, globs, paths } = scanFile(file, before);
        const pathEdits = [
            ...pathStringEdits(project, moves, strings, before),
            ...globEdits(project, moves, file, target, globs, before),
            ...relativePathEdits(moves, file, target, paths, before),
        ];
        edits.push(...pathEdits);
        for (const reference of modules) {
            // A stylesheet path is relative to the file even without a leading dot; the engine
            // works with the dotted form and writes the bare one back. A bare one that does not
            // resolve may be an alias or a preprocessor partial, which are not ours to guess.
            const bare = reference.css !== undefined && !reference.value.startsWith(".");
            const request = bare ? `./${reference.value}` : reference.value;
            const relative = reference.css !== undefined || reference.value.startsWith(".");
            const stylesheet = reference.css !== undefined;
            const resolved = resolver.resolve(file, request, reference.mode, stylesheet);
            if (!resolved) {
                if (file !== target && !bare && reference.value.startsWith(".")) {
                    throw new Error(`Cannot move ${file}: relative module ${JSON.stringify(reference.value)} cannot be resolved. No files were changed.`);
                }
                continue;
            }
            const expected = moves.files.get(path.resolve(resolved)) ?? path.resolve(resolved);
            const affected = expected !== path.resolve(resolved) || (file !== target && relative);
            // A path written without `.css` keeps being written without it.
            const [requested, suffix] = splitSuffix(request);
            const extensionless = stylesheet && !/\.css$/i.test(requested) && resolved.toLowerCase().endsWith(".css");
            const full = extensionless ? `${requested}.css${suffix}` : request;
            const respelled = affected ? replacement(project, full, target, expected, reference.mode) : full;
            const specifier = extensionless ? withoutCss(respelled) : respelled;
            const written = bare ? specifier.replace(/^\.\//, "") : specifier;
            checks.push({ specifier, expected, mode: reference.mode, rewritten: written !== reference.value, stylesheet });
            if (written !== reference.value) {
                edits.push({ start: reference.start, end: reference.end, text: textFor(reference, written, before) });
            }
        }
        if (edits.length === 0 && file === target) continue;
        let after = before;
        // Offsets come from the original AST. Apply all replacements backwards
        // once, keeping every other byte (comments, spacing, quote style) intact.
        for (const edit of edits.sort((a, b) => b.start - a.start)) {
            after = after.slice(0, edit.start) + edit.text + after.slice(edit.end);
        }
        changes.push({ file, target, before, after, edits: edits.length, checks, paths: pathEdits.map(edit => edit.expected) });
    }
    return { moves, changes };
}

function hasFrontmatterFence(bytes: Uint8Array): boolean {
    const offset = bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf ? 3 : 0;
    return bytes[offset] === 0x2d && bytes[offset + 1] === 0x2d && bytes[offset + 2] === 0x2d;
}

function textFor(reference: ModuleReference, written: string, before: string): string {
    if (reference.css) return quoteCssPath(written, reference.css.quote);
    if (reference.html) return quoteAttribute(written, before[reference.start]!);
    return quotePath(written, before[reference.start]!);
}

/** `./a/base.css?inline` -> `./a/base?inline` */
function withoutCss(specifier: string): string {
    const [base, suffix] = splitSuffix(specifier);
    return base.replace(/\.css$/i, "") + suffix;
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
        verifyPathStrings(change.target, change.paths, host ?? ts.sys);
        for (const check of change.checks) {
            if (!resolver.checkable(check.specifier)) {
                if (check.rewritten) unchecked++;
                continue;
            }
            const actual = resolver.resolve(change.target, check.specifier, check.mode, check.stylesheet);
            if (!actual || path.resolve(actual) !== check.expected) {
                throw new Error(`Moved module resolves incorrectly: ${change.target}: ${JSON.stringify(check.specifier)}; expected ${check.expected}, received ${actual ?? "unresolved"}. After the move another file or folder would answer that import, so nothing was changed; choose a target name that no other module answers.`);
            }
        }
    }
    return unchecked;
}
