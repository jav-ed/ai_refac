import { parseSync } from "oxc-parser";
import { frontmatterEnd, splitLines, type Line } from "./frontmatter";
import type { Region } from "./parse";

// MDX takes `import` and `export` at the start of a line, outside code fences,
// as JavaScript (ESM). Everything else in the file is Markdown and JSX.
const ESM_START = /^(?:import|export)[ \t{*]/;
const FENCE_OPEN = /^ {0,3}(`{3,}|~{3,})/;
const FENCE_CLOSE = /^ {0,3}(`{3,}|~{3,})$/;

/**
 * The ESM blocks of an `.mdx` file. A block runs to the next blank line, as in
 * MDX itself; when what has been read does not parse yet (a blank line inside
 * an object or a function), the block goes on to the next blank line. A block
 * that never parses is an error, not something to skip: a skipped import would
 * stay on the old path without a word.
 */
export function mdxRegions(file: string, source: string): Region[] {
    const lines = splitLines(source);
    const regions: Region[] = [];
    let fence = "";
    for (let index = frontmatterEnd(file, lines, true); index < lines.length; index++) {
        const { text } = lines[index]!;
        if (fence) {
            const closing = FENCE_CLOSE.exec(text.trimEnd())?.[1];
            if (closing && closing[0] === fence[0] && closing.length >= fence.length) fence = "";
            continue;
        }
        const opening = FENCE_OPEN.exec(text)?.[1];
        if (opening) { fence = opening; continue; }
        if (!ESM_START.test(text)) continue;
        const block = readBlock(file, source, lines, index);
        regions.push(block.region);
        index = block.last;
    }
    return regions;
}

function readBlock(file: string, source: string, lines: Line[], first: number): { region: Region; last: number } {
    const start = lines[first]!.start;
    for (let last = first; ; last++) {
        const atEnd = last === lines.length - 1;
        if (!atEnd && lines[last + 1]!.text.trim() !== "") continue;
        const text = source.slice(start, lines[last]!.start + lines[last]!.text.length);
        // The name only picks the language: ESM in MDX may hold JSX.
        const parsed = parseSync(`${file}.jsx`, text);
        if (!parsed.errors.length) return { region: { program: parsed.program, shift: start }, last };
        if (atEnd) {
            throw new Error(`Cannot parse the import/export block that starts at line ${first + 1} of ${file}:\n${parsed.errors.map(error => error.message).join("\n")}`);
        }
    }
}
