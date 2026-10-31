import * as fs from "node:fs";
import * as path from "node:path";
import type { Span } from "./Embedded/parse";
import { type MoveSet, targetOf } from "./moves";
import { type ProjectConfig } from "./project";
import { type PathEdit, quoteString, relativeSpelling } from "./strings";

/** One pattern of an `import.meta.glob` or `Astro.glob` call; patterns of one call share `call`. */
export interface GlobPattern extends Span { call: number; }

/** A pattern that starts in a folder of the project, with the folder before and after the moves. */
interface ResolvedGlob {
    pattern: GlobPattern;
    negated: boolean;
    /** What follows the folder, from its slash on (`/*.webp`), as written. Empty for a plain path. */
    rest: string;
    relative: boolean;
    before: string;
    after: string;
}

/**
 * The folder part of a glob pattern is a path and follows the moves; the rest of the pattern
 * (`/*.webp`) is kept as written. A pattern without glob characters is a path as a whole.
 */
function splitGlob(pattern: string): { folder: string; rest: string } {
    const at = pattern.search(/[*?[\]{}()!]/);
    if (at < 0) return { folder: pattern, rest: "" };
    const slash = pattern.lastIndexOf("/", at);
    return { folder: pattern.slice(0, slash), rest: pattern.slice(slash) };
}

/**
 * The patterns that start with `./`, `../` or `/` in an existing folder. Others (an alias, a package)
 * are not paths of ours.
 */
function resolveGlobs(project: ProjectConfig, moves: MoveSet, file: string, globs: GlobPattern[]): ResolvedGlob[] {
    const resolved: ResolvedGlob[] = [];
    for (const pattern of globs) {
        const negated = pattern.value.startsWith("!");
        const body = negated ? pattern.value.slice(1) : pattern.value;
        const relative = body.startsWith("./") || body.startsWith("../");
        if (!relative && !body.startsWith("/")) continue;
        let { folder, rest } = splitGlob(body);
        const resolve = (name: string) => relative ? path.resolve(path.dirname(file), name) : path.resolve(project.root, `.${name}`);
        // A name without glob characters that is no file, such as `!./skip.ts`, still sits in a folder.
        if (!rest && !fs.existsSync(resolve(folder))) {
            const slash = folder.lastIndexOf("/");
            rest = folder.slice(slash);
            folder = folder.slice(0, slash);
        }
        const before = resolve(folder);
        if (!fs.existsSync(before)) continue;
        resolved.push({ pattern, negated, rest, relative, before, after: targetOf(moves, before) ?? before });
    }
    return resolved;
}

/**
 * Glob patterns that start in a moved folder, and relative patterns of a file that moves. The
 * pattern keeps reading the same folder; the keys of its result are the paths it names, so they
 * change with it.
 */
export function globEdits(project: ProjectConfig, moves: MoveSet, file: string, target: string, globs: GlobPattern[], source: string): PathEdit[] {
    const edits: PathEdit[] = [];
    for (const { pattern: { start, end }, negated, rest, relative, before, after } of resolveGlobs(project, moves, file, globs)) {
        if (after === before && (!relative || path.dirname(target) === path.dirname(file))) continue;
        let written: string;
        if (relative) {
            written = relativeSpelling(path.dirname(target), after);
        } else {
            const from = path.relative(project.root, after).split(path.sep).join("/");
            written = from === "" ? "" : `/${from}`;
        }
        edits.push({ start, end, text: quoteString(`${negated ? "!" : ""}${written}${rest}`, source[start]!), expected: after });
    }
    return edits;
}

function matches(folder: string, rest: string, file: string): boolean {
    if (!rest) return file === folder;
    const inside = path.relative(folder, file);
    if (inside === ".." || inside.startsWith(`..${path.sep}`) || path.isAbsolute(inside)) return false;
    return new Bun.Glob(rest.slice(1)).match(inside.split(path.sep).join("/"));
}

/** The include pattern of a call that selects `file` in its folder `side`, unless an exclude pattern removes it. */
function selecting(call: ResolvedGlob[], side: "before" | "after", file: string): GlobPattern | undefined {
    if (call.some(item => item.negated && matches(item[side], item.rest, file))) return undefined;
    return call.find(item => !item.negated && matches(item[side], item.rest, file))?.pattern;
}

const MOST_NOTES = 20;

/**
 * What the moves change about the files that globs select, which no edit can repair: a file that
 * leaves the folder a glob reads (or lands in it) stops being (or starts being) in its result, and
 * a pattern that was rewritten has new keys. `calls` are the globs of every file, with the file.
 */
export function globNotes(project: ProjectConfig, moves: MoveSet, calls: { file: string; globs: GlobPattern[]; edited: boolean }[]): string[] {
    const shown = (file: string) => path.relative(project.root, file);
    const notes: string[] = [];
    let membership = 0;
    for (const { file, globs, edited } of calls) {
        const byCall = new Map<number, ResolvedGlob[]>();
        for (const item of resolveGlobs(project, moves, file, globs)) byCall.set(item.pattern.call, [...byCall.get(item.pattern.call) ?? [], item]);
        for (const call of byCall.values()) {
            for (const [from, to] of moves.files) {
                const left = selecting(call, "before", from);
                const joined = selecting(call, "after", to);
                if (Boolean(left) === Boolean(joined)) continue;
                if (++membership > MOST_NOTES) continue;
                notes.push(left
                    ? `${shown(from)} is selected by the glob "${left.value}" in ${shown(file)} and will not be after the move.`
                    : `${shown(to)} will be selected by the glob "${joined!.value}" in ${shown(file)}; ${shown(from)} was not.`);
            }
        }
        if (edited) notes.push(`The glob pattern in ${shown(file)} changed, and so did the keys of its result; code that writes a key by hand needs the same change.`);
    }
    if (membership > MOST_NOTES) notes.push(`...and ${membership - MOST_NOTES} more files that globs start or stop selecting.`);
    return notes;
}
