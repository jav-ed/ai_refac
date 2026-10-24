import { parseSync, type Program } from "oxc-parser";

/**
 * A piece of JavaScript or TypeScript that sits inside a file, already parsed.
 * `shift` is added to every offset the parse reports, so offsets always point
 * into the whole file and an edit can be made on the file's text as it is.
 */
export interface Region { program: Program; shift: number; }

/** A piece of a stylesheet that sits inside a file, with the offset of its first character. */
export interface TextRegion { text: string; shift: number; }

/** A string of a file with its place: `start`..`end` index the whole file's text. */
export interface Span { value: string; start: number; end: number; }

/**
 * What a file holds that names other files: the code, the stylesheets, the module specifiers
 * written as attribute values of the template (the span includes the quotes, as for a string in
 * code) and the paths relative to the file that its frontmatter holds (the span is the path alone).
 */
export interface Parts { code: Region[]; styles: TextRegion[]; specifiers: Span[]; paths: Span[]; }

/** Parses a whole script file. `file` decides the language (.ts, .tsx, .js, ...). */
export function parseCode(file: string, source: string): Program {
    const parsed = parseSync(file, source);
    if (parsed.errors.length) {
        throw new Error(`Cannot parse ${file}:\n${parsed.errors.map(error => error.message).join("\n")}`);
    }
    return parsed.program;
}
