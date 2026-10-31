import * as fs from "node:fs";
import * as path from "node:path";
import type ts from "typescript";
import type { Span } from "./Embedded/parse";
import { type MoveSet, targetOf } from "./moves";
import { type ProjectConfig } from "./project";

/** A string of a file with its span (the quotes included for a string of code). */
export type PathString = Span;

/**
 * Whether a string may name a file or folder of the project, written from the
 * project root (`src/a.png`, `./src/Content/blog/`). A cheap shape test: it
 * says nothing about whether the path exists. URLs, absolute paths, globs,
 * `..` paths (relative to some file, not the root) and text never pass.
 */
export function looksLikeRootPath(value: string): boolean {
    if (value.length > 400 || !value.includes("/") || value.startsWith("/")) return false;
    if (/[\s*?#{}[\]!\\:<>|"'`$]/.test(value) || value.includes("//")) return false;
    return !value.split("/").includes("..");
}

/** The edit for a string that names a moved path, and where the new path must exist. */
export interface PathEdit { start: number; end: number; text: string; expected: string; }

/**
 * Strings of the code that name a moved file or folder from the project root,
 * such as the `base` of a content collection or a font file in a config
 * object. A string counts only when it is exactly the path of something that
 * exists, so words and URLs are never touched. The spelling is kept: a leading
 * `./` and a trailing `/` stay. The path from the root does not depend on where
 * the file holding the string is, so moving that file changes nothing here.
 */
export function pathStringEdits(project: ProjectConfig, moves: MoveSet, strings: PathString[], source: string): PathEdit[] {
    const edits: PathEdit[] = [];
    for (const { value, start, end } of strings) {
        const current = path.resolve(project.root, value);
        if (!fs.existsSync(current)) continue;
        const target = targetOf(moves, current);
        if (target === undefined || target === current) continue;
        let written = path.relative(project.root, target).split(path.sep).join("/");
        if (value.startsWith("./")) written = `./${written}`;
        if (value.endsWith("/")) written += "/";
        edits.push({ start, end, text: quoteString(written, source[start]!), expected: target });
    }
    return edits;
}

/** How a path is spelled from `folder` to `target`: `./` or `../` first, as a pattern or a relative value needs. */
export function relativeSpelling(folder: string, target: string): string {
    const from = path.relative(folder, target).split(path.sep).join("/");
    return from === ".." || from.startsWith("../") ? from : from === "" ? "." : `./${from}`;
}

/**
 * Paths relative to the file that its text holds outside the code: the values of a page's
 * frontmatter (`image: ../assets/hero.webp`). They are paths only when they name something
 * that exists, so a value such as a canonical `../about` is never touched. They follow a moved
 * target, and a moved page keeps naming the same files from its new folder. The span is the
 * path alone; quotes around it stay.
 */
export function relativePathEdits(moves: MoveSet, file: string, target: string, paths: PathString[], source: string): PathEdit[] {
    const edits: PathEdit[] = [];
    for (const { value, start, end } of paths) {
        const current = path.resolve(path.dirname(file), value);
        if (!fs.existsSync(current)) continue;
        const moved = targetOf(moves, current) ?? current;
        if (moved === current && path.dirname(target) === path.dirname(file)) continue;
        const written = relativeSpelling(path.dirname(target), moved) + (value.endsWith("/") ? "/" : "");
        const quote = source[start - 1] === source[end] && /["']/.test(source[end] ?? "") ? source[end]! : "";
        edits.push({ start, end, text: yamlScalar(written, quote), expected: moved });
    }
    return edits;
}

/** A path in place of a YAML scalar's text: quoted as it was, or plain when it needs no quoting. */
function yamlScalar(value: string, quote: string): string {
    if (quote ? value.includes(quote) || value.includes("\\") : /#|: |^[^.]|\s$/.test(value)) {
        throw new Error(`Cannot write ${JSON.stringify(value)} in a YAML value${quote ? ` quoted with ${quote}` : ""}. Quote the value in the file first, or choose another name.`);
    }
    return value;
}

export function quoteString(value: string, quote: string): string {
    const escaped = JSON.stringify(value).slice(1, -1);
    if (quote === '"') return `"${escaped}"`;
    if (quote === "'") return `'${escaped.replaceAll("'", "\\'")}'`;
    if (quote === "`") return "`" + escaped.replaceAll("`", "\\`").replaceAll("${", "\\${") + "`";
    throw new Error(`Unsupported string delimiter ${JSON.stringify(quote)}`);
}

/** Throws unless every rewritten path exists in the project as it is after the moves (`host`: the disk, or the plan's model of it). */
export function verifyPathStrings(file: string, expected: string[], host: ts.ModuleResolutionHost): void {
    for (const target of expected) {
        if (host.fileExists(target) || host.directoryExists?.(target)) continue;
        throw new Error(`Moved path ${JSON.stringify(target)} in ${file} does not exist after the move. Nothing was changed.`);
    }
}
