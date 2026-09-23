import { expect, test } from "bun:test";
import { applyPlan } from "../../TypeScript/apply";
import { prepareMoves } from "../../TypeScript/moves";
import { planMoves } from "../../TypeScript/plan";
import { readProject } from "../../TypeScript/project";
import { fixture } from "./fixture";
import * as fs from "node:fs";
import * as path from "node:path";

test("malformed source syntax aborts the complete batch before mutation", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    f.write("src/b.ts", "export {value} from './a';");
    f.write("src/z.ts", "export const = ;");
    const before = f.snapshot();
    expect(() => f.move([["src/a.ts", "src/deep/a.ts"]])).toThrow("Cannot parse");
    expect(f.snapshot()).toEqual(before);
});

test("post-move resolution conflicts roll back source paths and all rewritten callers", () => {
    const f = fixture();
    f.write("src/a.tsx", "export const value = 1;");
    f.write("src/caller.ts", "export {value} from './a';");
    f.write("src/new/a.ts", "export const wrong = 2;");
    const before = f.snapshot();
    expect(() => f.move([["src/a.tsx", "src/new/a.tsx"]])).toThrow("original files restored");
    expect(f.snapshot()).toEqual(before);
});

test("concurrent edits are preserved and block applying a stale plan", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    f.write("src/caller.ts", "export {value} from './a';");
    const project = readProject(f.root);
    const plan = planMoves(project, prepareMoves(f.root, [["src/a.ts", "src/new/a.ts"]]));
    f.write("src/caller.ts", "// colleague edit\nexport {value} from './a';");
    const before = f.snapshot();
    expect(() => applyPlan(project, plan)).toThrow("Source changed while planning");
    expect(f.snapshot()).toEqual(before);
});

test("unresolved outgoing relative modules fail explicitly before movement", () => {
    const f = fixture();
    f.write("src/a.ts", "export * from './missing';");
    const before = f.snapshot();
    expect(() => f.move([["src/a.ts", "src/new/a.ts"]])).toThrow("cannot be resolved");
    expect(f.snapshot()).toEqual(before);
});

test("project references are rejected explicitly instead of silently omitting projects", () => {
    const f = fixture({ files: ["src/a.ts"], references: [{ path: "./child" }] });
    f.write("src/a.ts", "export const value = 1;");
    const before = f.snapshot();
    expect(() => f.move([["src/a.ts", "src/new/a.ts"]])).toThrow("project references");
    expect(f.snapshot()).toEqual(before);
});

test("locally bound require functions never cause unrelated string rewrites", () => {
    for (const source of [
        "const require = (x: string) => x; require('./a');",
        "function f(require: (s: string) => string) { return require('./a'); }",
        "const {require} = custom; require('./a');",
        "require = custom; require('./a');",
    ]) {
        const f = fixture();
        f.write("src/a.ts", "export const value = 1;");
        f.write("src/caller.ts", source);
        const before = f.snapshot();
        expect(() => f.move([["src/a.ts", "src/new/a.ts"]])).toThrow("locally bound require");
        expect(f.snapshot()).toEqual(before);
    }
});

test("dangling symlink targets are rejected without replacing the link", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    fs.symlinkSync("missing.ts", path.join(f.root, "src/link.ts"));
    const before = f.snapshot();
    expect(() => f.move([["src/a.ts", "src/link.ts"]])).toThrow("symlinks");
    expect(f.snapshot()).toEqual(before);
});

test("filesystem failure after one move restores paths, imports, and created directories", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    f.write("src/b.ts", "export {value} from './a';");
    const project = readProject(f.root);
    const plan = planMoves(project, prepareMoves(f.root, [["src/a.ts", "new/deep/a.ts"], ["src/b.ts", "blocked/b.ts"]]));
    f.write("blocked", "not a directory");
    const before = f.snapshot();
    expect(() => applyPlan(project, plan)).toThrow("original files restored");
    expect(f.snapshot()).toEqual(before);
    expect(f.exists("new")).toBe(false);
});

test("moving through a symlink is rejected before touching its destination", () => {
    const f = fixture(), outside = fixture();
    f.write("src/a.ts", "export const value = 1;");
    fs.symlinkSync(outside.root, path.join(f.root, "linked"), "dir");
    const before = f.snapshot(), outsideBefore = outside.snapshot();
    expect(() => f.move([["src/a.ts", "linked/a.ts"]])).toThrow("symlinks");
    expect(f.snapshot()).toEqual(before);
    expect(outside.snapshot()).toEqual(outsideBefore);
});

test("malformed config and escaping targets fail without partial edits", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    expect(() => f.move([["src/a.ts", "../outside.ts"]])).toThrow("inside project");
    f.write("tsconfig.json", "{ broken");
    const before = f.snapshot();
    expect(() => f.move([["src/a.ts", "src/new/a.ts"]])).toThrow();
    expect(f.snapshot()).toEqual(before);
});
