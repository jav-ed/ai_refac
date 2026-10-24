import type { Span } from "./parse";

export interface Line { text: string; start: number; }

export function splitLines(source: string): Line[] {
    const lines: Line[] = [];
    let start = 0;
    for (const raw of source.split("\n")) {
        lines.push({ text: raw.replace(/\r$/, ""), start });
        start += raw.length + 1;
    }
    return lines;
}

/**
 * The index of the first line after the frontmatter, or 0 when there is none. `.mdx` is strict:
 * a frontmatter that is never closed is an error. In `.md` a leading `---` that is never
 * closed is an ordinary line (a rule), so documents of that kind are left alone.
 */
export function frontmatterEnd(file: string, lines: Line[], strict: boolean): number {
    if (lines[0]?.text.replace(/^﻿/, "").trimEnd() !== "---") return 0;
    for (let index = 1; index < lines.length; index++) {
        if (lines[index]!.text.trimEnd() === "---") return index + 1;
    }
    if (strict) throw new Error(`Cannot read ${file}: the frontmatter that starts on line 1 is never closed with ---.`);
    return 0;
}

// One YAML line that holds a scalar: `key: value`, `- value`, `- key: value`, plain or quoted, with an optional comment.
const SCALAR = /^(?<lead>\s*(?:-\s+)*(?:[^\s:#"'[\]{}][^:#"']*:\s+)?)(?:"(?<double>[^"\\]*)"|'(?<single>[^']*)'|(?<plain>[^\s"'#[\]{}][^#]*?))\s*(?:#.*)?$/;

/**
 * The values of a frontmatter that are written as paths relative to the file (`./` or `../`),
 * such as the `image: ../assets/hero.webp` of a content collection. A span is the path alone,
 * without its quotes. A value that is no such path is not read; whether it names a file is the
 * planner's question.
 */
export function frontmatterPaths(file: string, source: string, strict: boolean): Span[] {
    const lines = splitLines(source);
    const end = frontmatterEnd(file, lines, strict);
    const found: Span[] = [];
    for (let index = 1; index < end - 1; index++) {
        const { text, start: lineStart } = lines[index]!;
        const groups = SCALAR.exec(text)?.groups;
        if (!groups) continue;
        const quoted = groups.double !== undefined || groups.single !== undefined;
        const value = groups.double ?? groups.single ?? groups.plain!;
        if (!/^\.{1,2}\//.test(value)) continue;
        const start = lineStart + groups.lead!.length + (quoted ? 1 : 0);
        found.push({ value, start, end: start + value.length });
    }
    return found;
}
