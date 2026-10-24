import { expect, test } from "bun:test";
import { collectImports } from "../../TypeScript/imports";
import { fixture } from "./fixture";

const PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler", baseUrl: ".", paths: { "@scripts/*": ["src/scripts/*"] } }, include: ["src/**/*"] };

// Verified against `astro build`: only a script whose single attribute is `src` is bundled.
test("a <script src> is an import when src is its only attribute, and a URL otherwise", () => {
    const source = [
        '<script src="../scripts/a.ts"></script>',
        "<script src='@scripts/b.ts'></script>",
        '<script type="module" src="../scripts/c.ts"></script>',
        '<script is:inline src="../scripts/d.ts"></script>',
        '<script async src="../scripts/e.ts"></script>',
        '<script type="module"></script>',
        '<img src="../assets/a.png" /><link rel="stylesheet" href="../assets/a.css" />',
        "",
    ].join("\n");
    const references = collectImports("/p/src/x.astro", source);
    expect(references.map(reference => reference.value)).toEqual(["../scripts/a.ts", "@scripts/b.ts"]);
    for (const reference of references) expect(source.slice(reference.start + 1, reference.end - 1)).toBe(reference.value);
});

test("moving a script rewrites the src that bundles it, its quotes kept, and nothing else", () => {
    const f = fixture(PROJECT);
    f.write("src/scripts/a.ts", 'console.log("a");\n');
    f.write("src/scripts/b.ts", 'console.log("b");\n');
    f.write("src/pages/index.astro", [
        '<script src="../scripts/a.ts"></script>', "<script src='@scripts/b.ts'></script>",
        '<script type="module" src="../scripts/a.ts"></script>', "",
    ].join("\n"));
    f.move([["src/scripts/a.ts", "src/scripts/core/a.ts"], ["src/scripts/b.ts", "src/scripts/core/b.ts"]]);
    expect(f.read("src/pages/index.astro")).toBe([
        '<script src="../scripts/core/a.ts"></script>', "<script src='@scripts/core/b.ts'></script>",
        '<script type="module" src="../scripts/a.ts"></script>', "",
    ].join("\n"));
});

test("moving the page rewrites a relative script src, an alias one stays", () => {
    const f = fixture(PROJECT);
    f.write("src/scripts/a.ts", 'console.log("a");\n');
    f.write("src/pages/index.astro", '<script src="../scripts/a.ts"></script>\n<script src="@scripts/a.ts"></script>\n');
    f.move([["src/pages/index.astro", "src/pages/deep/index.astro"]]);
    expect(f.read("src/pages/deep/index.astro")).toBe('<script src="../../scripts/a.ts"></script>\n<script src="@scripts/a.ts"></script>\n');
});

test("a script src that cannot be written in an attribute is an error and nothing changes", () => {
    const f = fixture(PROJECT);
    f.write("src/scripts/a.ts", 'console.log("a");\n');
    f.write("src/pages/index.astro", '<script src="../scripts/a.ts"></script>\n');
    const before = f.snapshot();
    expect(() => f.move([["src/scripts/a.ts", "src/scripts/a&b.ts"]])).toThrow(/HTML attribute/);
    expect(f.snapshot()).toEqual(before);
});
