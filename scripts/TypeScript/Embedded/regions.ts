import { astroParts } from "./astro";
import { frontmatterPaths } from "./frontmatter";
import { mdxRegions } from "./mdx";
import { parseCode, type Parts } from "./parse";

/**
 * What `file` holds that names other files. Script files: all of it is code.
 * Components: only the scripts and blocks inside them are code, and `.astro`
 * `<style>` blocks are stylesheets. A `.css` file is one stylesheet. A page
 * (`.mdx`, `.md`) names files in its frontmatter. Offsets of every part point
 * into the whole file.
 */
export function partsOf(file: string, source: string): Parts {
    if (file.endsWith(".astro")) return astroParts(file, source);
    if (file.endsWith(".mdx")) return { code: mdxRegions(file, source), styles: [], specifiers: [], paths: frontmatterPaths(file, source, true) };
    if (file.endsWith(".md")) return { code: [], styles: [], specifiers: [], paths: frontmatterPaths(file, source, false) };
    if (file.endsWith(".css")) return { code: [], styles: [{ text: source, shift: 0 }], specifiers: [], paths: [] };
    return { code: [{ program: parseCode(file, source), shift: 0 }], styles: [], specifiers: [], paths: [] };
}
