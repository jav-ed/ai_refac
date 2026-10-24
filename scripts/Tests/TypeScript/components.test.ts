import { expect, test } from "bun:test";
import { cssReferences } from "../../TypeScript/Embedded/css";
import { collectImports } from "../../TypeScript/imports";
import { readProject } from "../../TypeScript/project";
import { componentUsers } from "../../TypeScript/users";
import { fixture } from "./fixture";

// Every reference must point at exactly its quoted path in the whole file's text.
function expectExactSpans(source: string, references: { value: string; start: number; end: number }[]) {
    for (const reference of references) expect(source.slice(reference.start, reference.end)).toMatch(new RegExp(`^["'\`]${reference.value}["'\`]$`));
}

test("an .astro file names modules in its frontmatter and in scripts without attributes only", () => {
    const source = [
        "---", 'import A from "../a";', 'const fence = "--- not a fence";', "---", "",
        '<img src="../assets/logo.svg" />',
        "<script>", '  import { one } from "../lib/one";', "</script>",
        "<script is:inline>", '  import { two } from "../lib/two";', "</script>",
        '<script type="module">', '  import { three } from "../lib/three";', "</script>",
        "<script define:vars={{ a: 1 }}>", '  import { four } from "../lib/four";', "</script>",
        '{cond && <script>import { five } from "../lib/five";</script>}',
        '<!-- <script>import { six } from "../lib/six";</script> -->',
    ].join("\n");
    const references = collectImports("/p/src/x.astro", source);
    expect(references.map(reference => reference.value)).toEqual(["../a", "../lib/one", "../lib/five"]);
    expectExactSpans(source, references);
});

test("offsets stay exact after text that takes more than one UTF-16 unit or byte", () => {
    const astro = '---\n// Überschrift, بسم الله, 日本語, 😀\nimport A from "../a";\n---\n<h1>Grüße 😀</h1>\n<script>\n  // ä ö ü 😀\n  import { one } from "../lib/one";\n</script>\n';
    expectExactSpans(astro, collectImports("/p/x.astro", astro));
    const mdx = 'import A from "../a";\n\nÜberschrift 😀 بسم الله\n\nexport { b } from "../lib/b";\n';
    const references = collectImports("/p/x.mdx", mdx);
    expect(references.map(reference => reference.value)).toEqual(["../a", "../lib/b"]);
    expectExactSpans(mdx, references);
});

test("an .mdx file names modules in its import and export blocks, not in frontmatter, code fences or prose", () => {
    const source = [
        "---", "title: import x from 'not-an-import'", "---", "",
        'import Card from "../components/Card.astro";', 'import { a } from "../lib/a";', "",
        "# A heading about import", "",
        "```js", 'import fenced from "./fenced";', "```", "",
        // A blank line inside an object does not end the block while it is incomplete.
        "export const meta = {", "", '  title: "blank line inside",', "};", "",
        "Some prose that mentions import and export.", "  import indented from './indented';", "",
        'export { b } from "../lib/b";',
    ].join("\n");
    const references = collectImports("/p/x.mdx", source);
    expect(references.map(reference => reference.value)).toEqual(["../components/Card.astro", "../lib/a", "../lib/b"]);
    expectExactSpans(source, references);
});

test("code that cannot be read is an error that names the file, never a skipped import", () => {
    expect(() => collectImports("/p/x.mdx", 'import { from "x";\n')).toThrow(/Cannot parse the import\/export block that starts at line 1 of \/p\/x\.mdx/);
    expect(() => collectImports("/p/x.mdx", "---\ntitle: never closed\n")).toThrow(/frontmatter .* never closed/);
    expect(() => collectImports("/p/x.astro", "---\nconst = ;\n---\n<p></p>\n")).toThrow(/Cannot parse \/p\/x\.astro/);
});

const PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler", baseUrl: ".", paths: { "@lib/*": ["src/lib/*"], "@components/*": ["src/components/*"] } }, include: ["src/**/*"] };

test("moving a module updates its .astro and .mdx importers, aliases included, and leaves inline scripts alone", () => {
    const f = fixture(PROJECT);
    f.write("src/lib/format.ts", "export const formatDate = (date: Date) => date.toISOString();\n");
    f.write("src/components/Card.astro", [
        "---", 'import { formatDate } from "../lib/format";', "---", "<p>{formatDate(new Date())}</p>",
        "<script>", '  import { formatDate } from "@lib/format";', "</script>",
        "<script is:inline>", '  import("../lib/format");', "</script>", "",
    ].join("\n"));
    f.write("src/content/post.mdx", 'import { formatDate } from "../lib/format";\n\nWritten on {formatDate(new Date())}.\n');
    f.move([["src/lib/format.ts", "src/lib/text/format.ts"]]);
    expect(f.read("src/components/Card.astro")).toBe([
        "---", 'import { formatDate } from "../lib/text/format";', "---", "<p>{formatDate(new Date())}</p>",
        "<script>", '  import { formatDate } from "@lib/text/format";', "</script>",
        "<script is:inline>", '  import("../lib/format");', "</script>", "",
    ].join("\n"));
    expect(f.read("src/content/post.mdx")).toBe('import { formatDate } from "../lib/text/format";\n\nWritten on {formatDate(new Date())}.\n');
});

test("moving an .astro component rewrites its own imports, bundled scripts included, and everything that imports it", () => {
    const f = fixture(PROJECT);
    f.write("src/lib/track.ts", "export const track = (event: string) => event;\n");
    f.write("src/assets/logo.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/components/Badge.astro", "<span><slot /></span>\n");
    f.write("src/components/Card.astro", [
        "---", 'import logo from "../assets/logo.svg";', 'import Badge from "./Badge.astro";', 'import { track } from "@lib/track";', "---",
        "<Badge>{logo.src}</Badge>", "<script>", '  import { track } from "../lib/track";', "</script>", "",
    ].join("\n"));
    f.write("src/pages/index.astro", '---\nimport Card from "../components/Card.astro";\nimport Same from "@components/Card.astro";\n---\n<Card /><Same />\n');
    f.write("src/content/post.mdx", 'import Card from "../components/Card.astro";\n\n<Card />\n');
    f.move([["src/components/Card.astro", "src/components/ui/Card.astro"]]);
    // Relative imports follow the new place; the alias does not depend on it.
    expect(f.read("src/components/ui/Card.astro")).toBe([
        "---", 'import logo from "../../assets/logo.svg";', 'import Badge from "../Badge.astro";', 'import { track } from "@lib/track";', "---",
        "<Badge>{logo.src}</Badge>", "<script>", '  import { track } from "../../lib/track";', "</script>", "",
    ].join("\n"));
    expect(f.read("src/pages/index.astro")).toBe('---\nimport Card from "../components/ui/Card.astro";\nimport Same from "@components/ui/Card.astro";\n---\n<Card /><Same />\n');
    expect(f.read("src/content/post.mdx")).toBe('import Card from "../components/ui/Card.astro";\n\n<Card />\n');
    expect(f.exists("src/components/Card.astro")).toBe(false);
});

test("moving a folder of components moves its images along and fixes the importers", () => {
    const f = fixture(PROJECT);
    f.write("src/components/hero/hero.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/components/hero/Hero.astro", '---\nimport art from "./hero.svg";\n---\n<img src={art.src} />\n');
    f.write("src/pages/index.astro", '---\nimport Hero from "../components/hero/Hero.astro";\n---\n<Hero />\n');
    f.move([["src/components/hero", "src/sections/hero"]]);
    expect(f.read("src/pages/index.astro")).toBe('---\nimport Hero from "../sections/hero/Hero.astro";\n---\n<Hero />\n');
    expect(f.read("src/sections/hero/Hero.astro")).toBe('---\nimport art from "./hero.svg";\n---\n<img src={art.src} />\n');
    expect(f.exists("src/sections/hero/hero.svg")).toBe(true);
});

test("moving a document, video or archive that code imports fixes the import like any other asset", () => {
    const f = fixture(PROJECT);
    for (const name of ["manual.pdf", "intro.mp4", "kit.zip"]) f.write(`src/files/${name}`, "bytes");
    f.write("src/pages/download.astro", '---\nimport manual from "../files/manual.pdf";\nimport intro from "../files/intro.mp4";\nimport kit from "../files/kit.zip?url";\n---\n<a href={manual}>{intro}{kit}</a>\n');
    f.move([["src/files/manual.pdf", "src/downloads/manual.pdf"], ["src/files/intro.mp4", "src/downloads/intro.mp4"], ["src/files/kit.zip", "src/downloads/kit.zip"]]);
    expect(f.read("src/pages/download.astro")).toBe('---\nimport manual from "../downloads/manual.pdf";\nimport intro from "../downloads/intro.mp4";\nimport kit from "../downloads/kit.zip?url";\n---\n<a href={manual}>{intro}{kit}</a>\n');
});

// The Markdown driver moves pages and pictures; the engine only fixes the code that uses them, before that move.
test("an external move fixes the importers and the page's own imports but renames nothing", () => {
    const f = fixture(PROJECT);
    f.write("src/assets/hero.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/components/Card.astro", "<p><slot /></p>\n");
    f.write("src/pages/index.astro", '---\nimport art from "../assets/hero.svg";\nimport Post from "../content/post.mdx";\n---\n<img src={art.src} /><Post />\n');
    f.write("src/content/post.mdx", 'import Card from "../components/Card.astro";\n\n<Card />\n');
    f.move([], [["src/assets/hero.svg", "src/images/hero.svg"], ["src/content/post.mdx", "src/content/2026/post.mdx"]]);
    // Importers follow the new places, the page's own import follows its new folder...
    expect(f.read("src/pages/index.astro")).toBe('---\nimport art from "../images/hero.svg";\nimport Post from "../content/2026/post.mdx";\n---\n<img src={art.src} /><Post />\n');
    expect(f.read("src/content/post.mdx")).toBe('import Card from "../../components/Card.astro";\n\n<Card />\n');
    // ...and the files are where they were: the other tool moves them.
    expect(f.exists("src/assets/hero.svg")).toBe(true);
    expect(f.exists("src/images/hero.svg")).toBe(false);
    expect(f.exists("src/content/2026/post.mdx")).toBe(false);
});

test("external moves ride along with a move of the engine's own", () => {
    const f = fixture(PROJECT);
    f.write("src/lib/format.ts", "export const formatDate = (date: Date) => date.toISOString();\n");
    f.write("src/assets/hero.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/pages/index.astro", '---\nimport art from "../assets/hero.svg";\nimport { formatDate } from "../lib/format";\n---\n<img src={art.src} />{formatDate(new Date())}\n');
    f.move([["src/lib/format.ts", "src/lib/text/format.ts"]], [["src/assets/hero.svg", "src/images/hero.svg"]]);
    expect(f.read("src/pages/index.astro")).toBe('---\nimport art from "../images/hero.svg";\nimport { formatDate } from "../lib/text/format";\n---\n<img src={art.src} />{formatDate(new Date())}\n');
    expect(f.exists("src/lib/text/format.ts")).toBe(true);
    expect(f.exists("src/assets/hero.svg")).toBe(true);
});

test("a failed move puts back the edits made for an external move too", () => {
    const f = fixture(PROJECT);
    f.write("src/assets/hero.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/pages/index.astro", '---\nimport art from "../assets/hero.svg";\n---\n<img src={art.src} />\n');
    // `./a` answers with a.ts from the new folder, not with the moved a.tsx: the move is refused.
    f.write("src/a.tsx", "export const value = 1;\n");
    f.write("src/caller.ts", "export { value } from './a';\n");
    f.write("src/new/a.ts", "export const wrong = 2;\n");
    const before = f.snapshot();
    expect(() => f.move([["src/a.tsx", "src/new/a.tsx"]], [["src/assets/hero.svg", "src/images/hero.svg"]])).toThrow(/original files restored/);
    expect(f.snapshot()).toEqual(before);
});

// A rename of the language server cannot read inside component files; these are the ones it would leave behind.
test("component users of a symbol are the .astro and .mdx files that import from its files and mention it", () => {
    const f = fixture(PROJECT);
    f.write("src/lib/format.ts", "export const formatDate = (date: Date) => date.toISOString();\nexport const other = 1;\n");
    f.write("src/lib/unrelated.ts", "export const formatDate = () => 'another module with the same name';\n");
    // Uses it: the import names it, and a template calls it.
    f.write("src/pages/index.astro", '---\nimport { formatDate } from "../lib/format";\n---\n<p>{formatDate(new Date())}</p>\n');
    // Imports from the file but never writes the name: a rename cannot leave it behind.
    f.write("src/pages/other.astro", '---\nimport { other } from "../lib/format";\n---\n<p>{other}</p>\n');
    // Writes the name but imports it from another module.
    f.write("src/pages/elsewhere.astro", '---\nimport { formatDate } from "../lib/unrelated";\n---\n<p>{formatDate()}</p>\n');
    // A longer name that contains it is another name.
    f.write("src/pages/longer.astro", '---\nimport { other } from "@lib/format";\nconst formatDateTime = other;\n---\n<p>{formatDateTime}</p>\n');
    f.write("src/content/post.mdx", 'import { formatDate } from "@lib/format";\n\nWritten {formatDate(new Date())}.\n');
    const users = componentUsers(readProject(f.root), "formatDate", [`${f.root}/src/lib/format.ts`], ["astro", "mdx"]);
    expect(users.map(file => file.slice(f.root.length + 1)).sort()).toEqual(["src/content/post.mdx", "src/pages/index.astro"]);
    // Only the kinds the language server cannot read are asked for.
    expect(componentUsers(readProject(f.root), "formatDate", [`${f.root}/src/lib/format.ts`], ["mdx"]).map(file => file.slice(f.root.length + 1))).toEqual(["src/content/post.mdx"]);
});

test("a stylesheet names local files in url() and @import, and nothing else", () => {
    const css = [
        '/* url("../in-comment.png") */',
        '.a { background: url("../a.png") no-repeat; }',
        ".b { background: URL( '../b.png' ); }",
        ".c { background: url(../c.png); }",
        ".d { background: url(d.png); }",
        '@import "./e.css";',
        "@import url('./f.css') screen;",
        '.g::after { content: "url(../in-string.png)"; }',
        ".h { background: url(data:image/png;base64,AAAA), url(/site-root.png), url(https://x.test/a.png), url(#gradient); }",
        ".i { background: url($asset-path), url(var(--x)); }",
        ".j { border-image: myurl(../not-a-function.png); }",
    ].join("\n");
    const found = cssReferences(css);
    expect(found.map(reference => reference.value)).toEqual(["../a.png", "../b.png", "../c.png", "d.png", "./e.css", "./f.css"]);
    expect(found.map(reference => reference.quote)).toEqual(['"', "'", "", "", '"', "'"]);
    for (const reference of found) expect(css.slice(reference.start, reference.end)).toBe(reference.value);
    expect(() => cssReferences('.k { background: url("a\\ b.png"); }')).toThrow(/CSS path with an escape/);
});

const STYLE_PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler" }, include: ["src/**/*"] };

test("moving an .astro component rewrites the paths in its <style> blocks, quotes and bare forms kept", () => {
    const f = fixture(STYLE_PROJECT);
    f.write("src/assets/logo.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/assets/icon.png", "png");
    f.write("src/styles/base.css", ".base {}\n");
    f.write("src/components/Card.astro", [
        "<div class='card'>Card</div>", "<style>", '  .card { background: url("../assets/logo.svg") no-repeat; }',
        "  .icon { background: url(../assets/icon.png); }", '  @import "../styles/base.css";', "</style>",
        '<style is:global lang="css">', "  .bare { background: url(missing-elsewhere.png); }", "</style>", "",
    ].join("\n"));
    f.move([["src/components/Card.astro", "src/components/ui/Card.astro"]]);
    expect(f.read("src/components/ui/Card.astro")).toBe([
        "<div class='card'>Card</div>", "<style>", '  .card { background: url("../../assets/logo.svg") no-repeat; }',
        "  .icon { background: url(../../assets/icon.png); }", '  @import "../../styles/base.css";', "</style>",
        '<style is:global lang="css">', "  .bare { background: url(missing-elsewhere.png); }", "</style>", "",
    ].join("\n"));
});

test("moving a picture or a stylesheet fixes the paths in <style> blocks and in other stylesheets", () => {
    const f = fixture(STYLE_PROJECT);
    f.write("src/assets/logo.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/styles/base.css", ".base { background: url(../assets/logo.svg); }\n");
    f.write("src/styles/theme.css", '@import "./base.css";\n.t { background: url("../assets/logo.svg"); }\n');
    f.write("src/components/Card.astro", '---\nimport "../styles/theme.css";\n---\n<div />\n<style>\n  .c { background: url(\'../assets/logo.svg\'); }\n</style>\n');
    f.move([["src/assets/logo.svg", "src/images/brand/logo.svg"]]);
    expect(f.read("src/styles/base.css")).toBe(".base { background: url(../images/brand/logo.svg); }\n");
    expect(f.read("src/styles/theme.css")).toBe('@import "./base.css";\n.t { background: url("../images/brand/logo.svg"); }\n');
    expect(f.read("src/components/Card.astro")).toContain("url('../images/brand/logo.svg')");
    f.move([["src/styles/base.css", "src/styles/core/base.css"]]);
    // The importers follow the moved stylesheet, and its own path to the picture follows its new folder.
    expect(f.read("src/styles/theme.css")).toContain('@import "./core/base.css";');
    expect(f.read("src/styles/core/base.css")).toBe(".base { background: url(../../images/brand/logo.svg); }\n");
});

test("a path that cannot be written in an unquoted url() is an error, not a broken stylesheet", () => {
    const f = fixture(STYLE_PROJECT);
    f.write("src/assets/logo.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/styles/base.css", ".base { background: url(../assets/logo.svg); }\n");
    const before = f.snapshot();
    expect(() => f.move([["src/assets/logo.svg", "src/assets/my logo.svg"]])).toThrow(/unquoted url\(\)/);
    expect(f.snapshot()).toEqual(before);
});

test("a bare stylesheet path stays bare when the picture moves below it", () => {
    const f = fixture(STYLE_PROJECT);
    f.write("src/components/card.png", "png");
    f.write("src/components/Card.astro", "<div />\n<style>\n  .c { background: url(card.png); }\n</style>\n");
    f.move([["src/components/card.png", "src/components/img/card.png"]]);
    expect(f.read("src/components/Card.astro")).toContain("url(img/card.png)");
});

test("a stylesheet imported without .css follows the stylesheet, still written without it", () => {
    const f = fixture(STYLE_PROJECT);
    f.write("src/styles/base.css", ".base { background: url(../assets/logo.svg); }\n");
    f.write("src/assets/logo.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
    f.write("src/styles/pages/posts.css", '@import "../base";\n@import url("../base?inline");\n');
    f.move([["src/styles/base.css", "src/styles/core/base.css"]]);
    expect(f.read("src/styles/pages/posts.css")).toBe('@import "../core/base";\n@import url("../core/base?inline");\n');
    // The stylesheet's own path to the picture follows its folder.
    expect(f.read("src/styles/core/base.css")).toBe(".base { background: url(../../assets/logo.svg); }\n");
    f.move([["src/styles/pages/posts.css", "src/styles/posts.css"]]);
    expect(f.read("src/styles/posts.css")).toBe('@import "./core/base";\n@import url("./core/base?inline");\n');
});
