import { parse } from "@astrojs/compiler-rs";
import type { Program } from "oxc-parser";
import type { Parts, Span } from "./parse";

/**
 * What an `.astro` file holds that names other files: the code (the
 * frontmatter and the scripts Astro bundles) and the `<style>` blocks. The parser is Astro's own (Rust, the one `astro
 * build` uses), so strings, comments and expressions cannot fool it, and its
 * offsets are indices into the file's text.
 *
 * Astro bundles a `<script>` that has no attributes, or only `src`, and reads
 * the `src` as an import of the file. Every other script (`is:inline`, `type`,
 * `async`, `define:vars`, `lang`) reaches the browser as written, so a path in
 * it is a URL of the page, not a path from this file, and moving a file must not
 * touch it. The other `src`, `href` and `poster` attributes of the template are
 * URLs too: Astro copies them as they are.
 */
export function astroParts(file: string, source: string): Parts {
    const { ast, diagnostics } = parse(source);
    const errors = diagnostics.filter(diagnostic => diagnostic.severity === "error");
    if (errors.length) {
        const lines = errors.map(error => `${error.text}${error.labels[0] ? ` (line ${error.labels[0].line})` : ""}`);
        throw new Error(`Cannot parse ${file}:\n${lines.join("\n")}`);
    }
    const programs: Program[] = [];
    if (ast.frontmatter) programs.push(ast.frontmatter.program);
    const specifiers: Span[] = [];
    bundledScripts(ast.body, programs, specifiers);
    return { code: programs.map(program => ({ program, shift: 0 })), styles: styleBlocks(ast.body, source), specifiers, paths: [] };
}

interface Attribute {
    name?: { name?: string };
    value?: { type?: string; value?: unknown; start: number; end: number } | null;
}
interface Element {
    type?: string;
    openingElement?: { name?: { name?: string }; attributes: Attribute[] };
    children?: { type: string; program: Program }[];
}

/**
 * Every `<script>` of the template Astro bundles, wherever it sits (inside an
 * expression too): the programs of those with no attribute, and the `src` of
 * those with only `src`.
 */
function bundledScripts(node: unknown, programs: Program[], specifiers: Span[]): void {
    if (Array.isArray(node)) {
        for (const item of node) bundledScripts(item, programs, specifiers);
        return;
    }
    if (!node || typeof node !== "object") return;
    const element = node as Element;
    if (element.type === "AstroScript") return;
    if (element.type === "JSXElement") {
        const script = element.children?.find(child => child.type === "AstroScript");
        const attributes = element.openingElement?.attributes ?? [];
        if (script && attributes.length === 0) programs.push(script.program);
        const source = attributes[0]?.value;
        if (script && attributes.length === 1 && attributes[0]!.name?.name === "src" && source?.type === "Literal" && typeof source.value === "string") {
            specifiers.push({ value: source.value, start: source.start, end: source.end });
        }
    }
    for (const value of Object.values(node)) bundledScripts(value, programs, specifiers);
}

interface StyleElement {
    type?: string;
    openingElement?: { name?: { name?: string } };
    children?: { type: string; start: number; end: number }[];
}

/** The text of every `<style>` of the template, global, scoped or in an SVG; its offset is where the text starts in the file. */
function styleBlocks(node: unknown, source: string, found: { text: string; shift: number }[] = []): { text: string; shift: number }[] {
    if (Array.isArray(node)) {
        for (const item of node) styleBlocks(item, source, found);
        return found;
    }
    if (!node || typeof node !== "object") return found;
    const element = node as StyleElement;
    if (element.type === "JSXElement" && element.openingElement?.name?.name === "style") {
        for (const child of element.children ?? []) {
            if (child.type === "JSXText") found.push({ text: source.slice(child.start, child.end), shift: child.start });
        }
        return found;
    }
    for (const value of Object.values(node)) styleBlocks(value, source, found);
    return found;
}
