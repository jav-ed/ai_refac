import { expect, test } from "bun:test";
import { scanFile } from "../../TypeScript/imports";
import { fixture } from "./fixture";

const PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler", allowJs: true }, include: ["**/*"] };

const PAGE = [
    "---",
    "title: ../not a path, it is a title",
    "seo:",
    "  image:",
    "    src: ../../assets/hero.webp",
    "    alt: ../../assets/hero.webp is the picture",
    '  og: "../../assets/hero.webp"',
    "  twitter: '../../assets/hero.webp'   # a comment",
    "gallery:",
    "  - ../../assets/hero.webp",
    "  - https://example.com/../x.webp",
    "  - /public/hero.webp",
    "canonical: ../about",
    "---",
    "",
    "Body text with ../../assets/hero.webp in it.",
    "",
].join("\n");

// The reader returns what is written like a relative path; the planner decides which of them name a file.
test("the values of a frontmatter that start like a relative path are found, with the exact span of the path", () => {
    const { paths } = scanFile("/p/src/content/a/post.mdx", PAGE);
    expect(paths.map(item => item.value)).toEqual([
        "../not a path, it is a title", "../../assets/hero.webp", "../../assets/hero.webp is the picture",
        "../../assets/hero.webp", "../../assets/hero.webp", "../../assets/hero.webp", "../about",
    ]);
    for (const item of paths) expect(PAGE.slice(item.start, item.end)).toBe(item.value);
});

test("an .md page is read the same way, and a leading --- that never closes is no frontmatter there", () => {
    expect(scanFile("/p/post.md", PAGE).paths).toHaveLength(7);
    expect(scanFile("/p/notes.md", "---\n\nsrc: ../a.png\n").paths).toEqual([]);
    expect(() => scanFile("/p/notes.mdx", "---\n\nsrc: ../a.png\n")).toThrow(/never closed/);
});

function site() {
    const f = fixture(PROJECT);
    f.write("src/assets/hero.webp", "image");
    f.write("src/content/a/post.mdx", PAGE);
    f.write("src/content/a/note.md", PAGE);
    f.write("src/about.txt", "about");
    return f;
}

test("moving a picture that frontmatter names fixes every value that names it, the quotes kept", () => {
    const f = site();
    f.move([["src/assets/hero.webp", "src/assets/team/hero.webp"]]);
    for (const page of ["src/content/a/post.mdx", "src/content/a/note.md"]) {
        const text = f.read(page);
        expect(text).toContain("    src: ../../assets/team/hero.webp\n");
        expect(text).toContain('  og: "../../assets/team/hero.webp"\n');
        expect(text).toContain("  twitter: '../../assets/team/hero.webp'   # a comment\n");
        expect(text).toContain("  - ../../assets/team/hero.webp\n");
        // Text that only mentions the path stays as it is.
        expect(text).toContain("    alt: ../../assets/hero.webp is the picture\n");
        expect(text).toContain("Body text with ../../assets/hero.webp in it.");
    }
});

test("moving the page keeps its frontmatter naming the same files, and a value that names nothing stays", () => {
    const f = site();
    f.move([["src/content/a/post.mdx", "src/content/a/2024/post.mdx"]]);
    const text = f.read("src/content/a/2024/post.mdx");
    expect(text).toContain("    src: ../../../assets/hero.webp\n");
    expect(text).toContain('  og: "../../../assets/hero.webp"\n');
    expect(text).toContain("canonical: ../about\n");
    expect(text).toContain("title: ../not a path, it is a title\n");
});

test("a page and a picture that another tool moves are planned, not renamed", () => {
    const f = site();
    f.move([], [["src/assets/hero.webp", "src/assets/team/hero.webp"], ["src/content/a/post.mdx", "src/content/b/post.mdx"]]);
    expect(f.read("src/content/a/note.md")).toContain("    src: ../../assets/team/hero.webp\n");
    // The page is still where it was; its values already name the picture's new place from the new folder.
    expect(f.exists("src/content/a/post.mdx")).toBe(true);
    expect(f.read("src/content/a/post.mdx")).toContain("    src: ../../assets/team/hero.webp\n");
});

test("a new path that cannot be written in the YAML value is an error and nothing changes", () => {
    const f = site();
    const before = f.snapshot();
    expect(() => f.move([["src/assets/hero.webp", "src/assets/hero #1.webp"]])).toThrow(/YAML value/);
    expect(f.snapshot()).toEqual(before);
});

test("an .md file without frontmatter is never read as text, so its encoding cannot stop a move", () => {
    const f = site();
    f.write("src/notes.md", Buffer.from([0x52, 0xe9, 0x73, 0x75, 0x6d, 0xe9, 0x0a]));
    f.move([["src/assets/hero.webp", "src/assets/team/hero.webp"]]);
    expect(f.read("src/content/a/note.md")).toContain("../../assets/team/hero.webp");
});
