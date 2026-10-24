import { expect, test } from "bun:test";
import { fixture } from "./fixture";

const PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler", allowJs: true }, include: ["**/*"] };

function collections() {
    const f = fixture(PROJECT);
    f.write("src/Content/blog/post.mdx", "# Post\n");
    f.write("src/assets/fonts/a.woff2", "font");
    f.write("src/content.config.ts", [
        'export const blog = { base: "./src/Content/blog", pattern: "**/*.mdx" };',
        "export const fonts = ['src/assets/fonts/a.woff2'];",
        "export const tail = `src/Content/blog/`;",
        "",
    ].join("\n"));
    return f;
}

test("a string that is the root path of a moved folder or file follows the move, its spelling kept", () => {
    const f = collections();
    f.move([["src/Content/blog", "src/Content/articles"], ["src/assets/fonts/a.woff2", "src/assets/fonts/latin/a.woff2"]]);
    expect(f.read("src/content.config.ts")).toBe([
        'export const blog = { base: "./src/Content/articles", pattern: "**/*.mdx" };',
        "export const fonts = ['src/assets/fonts/latin/a.woff2'];",
        "export const tail = `src/Content/articles/`;",
        "",
    ].join("\n"));
});

test("a folder that contains the moved one changes the strings below it", () => {
    const f = collections();
    f.move([["src/Content", "content"]]);
    expect(f.read("src/content.config.ts")).toContain('"./content/blog"');
});

test("text, URLs, patterns, paths from a file and paths that do not exist are left alone", () => {
    const f = fixture(PROJECT);
    f.write("src/Content/blog/post.mdx", "# Post\n");
    const text = [
        'export const a = "src/Content/missing";', 'export const b = "https://example.com/src/Content/blog";',
        'export const c = "/src/Content/blog";', 'export const d = "../Content/blog";', 'export const e = "src/Content/*";',
        'export const f = "blog";', 'export const g = "src/Content/blog and more";', "",
    ].join("\n");
    f.write("src/config.ts", text);
    f.move([["src/Content/blog", "src/Content/articles"]]);
    expect(f.read("src/config.ts")).toBe(text);
});

test("an import specifier is fixed as an import, once, and the file that holds a root path can move", () => {
    const f = fixture(PROJECT);
    f.write("src/lib/a.ts", "export const a = 1;\n");
    f.write("src/config.ts", 'import { a } from "./lib/a";\nexport const root = "src/lib/a.ts";\nexport { a };\n');
    f.move([["src/lib/a.ts", "src/lib/core/a.ts"]]);
    expect(f.read("src/config.ts")).toBe('import { a } from "./lib/core/a";\nexport const root = "src/lib/core/a.ts";\nexport { a };\n');
    f.move([["src/config.ts", "src/other/config.ts"]]);
    // The root path does not depend on the folder of the file that holds it.
    expect(f.read("src/other/config.ts")).toContain('export const root = "src/lib/core/a.ts";');
});

test("an external folder move fixes the strings that name it and renames nothing", () => {
    const f = collections();
    f.move([], [["src/Content/blog", "src/Content/articles"]]);
    expect(f.read("src/content.config.ts")).toContain('base: "./src/Content/articles"');
    expect(f.exists("src/Content/blog/post.mdx")).toBe(true);
});
