import { expect, test } from "bun:test";
import { scanFile } from "../../TypeScript/imports";
import { fixture } from "./fixture";

const PROJECT = { compilerOptions: { module: "esnext", moduleResolution: "bundler", allowJs: true, baseUrl: ".", paths: { "@assets/*": ["src/assets/*"] } }, include: ["**/*"] };

test("the patterns of import.meta.glob and Astro.glob are found, with the exact span of their string", () => {
    const source = [
        "---",
        'const a = import.meta.glob("./posts/*.md");',
        'const b = import.meta.glob<{ default: string }>("../assets/*.svg", { eager: true, import: "default" });',
        "const c = import.meta.glob(['./x/*.ts', \"!./x/skip.ts\"]);",
        'const d = await Astro.glob("../pages/*.astro");',
        'const e = import.meta.glob("./based/*.ts", { base: "/src" });',
        'const f = other.glob("./not/*.ts");',
        "---",
        "",
    ].join("\n");
    const { globs } = scanFile("/p/src/x.astro", source);
    expect(globs.map(glob => glob.value)).toEqual(["./posts/*.md", "../assets/*.svg", "./x/*.ts", "!./x/skip.ts", "../pages/*.astro"]);
    for (const glob of globs) expect(source.slice(glob.start + 1, glob.end - 1)).toBe(glob.value);
});

function site() {
    const f = fixture(PROJECT);
    f.write("src/assets/team/a.webp", "a");
    f.write("src/icons/one.svg", "<svg/>");
    f.write("src/components/sibling.ts", "export {};\n");
    f.write("src/components/Team.astro", [
        "---",
        'const photos = import.meta.glob<{ default: string }>("../assets/team/*.webp", { eager: true });',
        'const icons = import.meta.glob("/src/icons/*.svg", { eager: true, import: "default" });',
        'const siblings = import.meta.glob(["./*.ts", "!./skip.ts"]);',
        "---",
        "<div />",
        "",
    ].join("\n"));
    return f;
}

test("a folder that a pattern starts in is followed when it moves, and the rest of the pattern is kept", () => {
    const f = site();
    f.move([["src/assets/team", "src/assets/people"], ["src/icons", "src/ui/icons"]]);
    const text = f.read("src/components/Team.astro");
    expect(text).toContain('import.meta.glob<{ default: string }>("../assets/people/*.webp", { eager: true })');
    expect(text).toContain('import.meta.glob("/src/ui/icons/*.svg", { eager: true, import: "default" })');
});

test("moving the file that holds a relative pattern keeps it reading the same folder", () => {
    const f = site();
    f.move([["src/components/Team.astro", "src/components/deep/Team.astro"]]);
    const text = f.read("src/components/deep/Team.astro");
    expect(text).toContain('"../../assets/team/*.webp"');
    // A root pattern does not depend on where the file is.
    expect(text).toContain('"/src/icons/*.svg"');
    expect(text).toContain('import.meta.glob(["../*.ts", "!../skip.ts"])');
});

test("patterns with an alias, a base option or a folder that does not exist are left alone", () => {
    const f = fixture(PROJECT);
    f.write("src/assets/team/a.webp", "a");
    const text = [
        'export const a = import.meta.glob("@assets/team/*.webp");',
        'export const b = import.meta.glob("./team/*.webp", { base: "/src/other" });',
        'export const c = import.meta.glob("/src/missing/*.webp");',
        "",
    ].join("\n");
    // The pattern of `b` would name the moved folder if its base option were ignored.
    f.write("src/assets/lib.ts", text);
    f.move([["src/assets/team", "src/assets/people"]]);
    expect(f.read("src/assets/lib.ts")).toBe(text);
});

test("a glob folder that an external move carries out is followed and nothing is renamed", () => {
    const f = site();
    f.move([], [["src/assets/team", "src/assets/people"]]);
    expect(f.read("src/components/Team.astro")).toContain('"../assets/people/*.webp"');
    expect(f.exists("src/assets/team/a.webp")).toBe(true);
});

test("a file that leaves the folder a glob reads, or lands in it, is reported, because no edit can keep it in the glob", () => {
    const f = site();
    f.write("src/other/b.webp", "b");
    const leaving = f.move([["src/assets/team/a.webp", "src/other/a.webp"]]);
    expect(leaving.notes).toEqual([
        'src/assets/team/a.webp is selected by the glob "../assets/team/*.webp" in src/components/Team.astro and will not be after the move.',
    ]);
    const joining = f.move([["src/other/b.webp", "src/assets/team/b.webp"]]);
    expect(joining.notes).toEqual([
        'src/assets/team/b.webp will be selected by the glob "../assets/team/*.webp" in src/components/Team.astro; src/other/b.webp was not.',
    ]);
});

test("a folder that moves with its glob, and a file that stays inside the pattern, report nothing", () => {
    const f = site();
    f.write("src/assets/team/more/c.webp", "c");
    f.write("src/other/skip.webp", "skip");
    f.write("src/components/Gallery.astro", '---\nconst all = import.meta.glob(["../assets/**/*.webp", "!../assets/hidden/*.webp"]);\n---\n');
    // The folder moves together with the pattern that reads it; the key warning is the only note.
    expect(f.move([["src/assets/team", "src/assets/people"]]).notes.filter(note => note.includes("selected"))).toEqual([]);
    // Inside the `**` pattern the file keeps being selected; the exclude does not apply to it.
    expect(f.move([["src/assets/people/more/c.webp", "src/assets/people/c.webp"]]).notes.filter(note => note.includes("Gallery.astro"))).toEqual([]);
    // An excluded file stays excluded when it is renamed; one level deeper the exclude no longer covers it.
    f.write("src/assets/hidden/h.webp", "h");
    expect(f.move([["src/assets/hidden/h.webp", "src/assets/hidden/h2.webp"]]).notes).toEqual([]);
    expect(f.move([["src/assets/hidden/h2.webp", "src/assets/hidden/deep/h2.webp"]]).notes).toEqual([
        'src/assets/hidden/deep/h2.webp will be selected by the glob "../assets/**/*.webp" in src/components/Gallery.astro; src/assets/hidden/h2.webp was not.',
    ]);
});

test("a rewritten pattern is reported once per file, with the reason", () => {
    const f = site();
    const { notes } = f.move([["src/assets/team", "src/assets/people"]]);
    expect(notes).toEqual(["The glob pattern in src/components/Team.astro changed, and so did the keys of its result; code that writes a key by hand needs the same change."]);
});
