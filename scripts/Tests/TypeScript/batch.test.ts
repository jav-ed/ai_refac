import { expect, test } from "bun:test";
import { fixture } from "./fixture";

test("a batch plans cyclic dependencies against final locations once", () => {
    const f = fixture();
    f.write("src/a.ts", "export {b} from './b'; export const a = 1;");
    f.write("src/b.ts", "export {a} from './a'; export const b = 2;");
    f.write("src/caller.ts", "export {a} from './a'; export {b} from './b';");
    f.move([["src/a.ts", "src/first/a.ts"], ["src/b.ts", "src/second/b.ts"]]);
    expect(f.read("src/first/a.ts")).toContain("'../second/b'");
    expect(f.read("src/second/b.ts")).toContain("'../first/a'");
    expect(f.read("src/caller.ts")).toBe("export {a} from './first/a'; export {b} from './second/b';");
});

test("tsconfig exclusions are respected and included callers beyond src are updated", () => {
    const f = fixture({ compilerOptions: { module: "esnext", moduleResolution: "bundler" }, include: ["src/**/*", "tools/**/*"], exclude: ["src/excluded.ts"] });
    f.write("src/a.ts", "export const value = 1;");
    f.write("tools/use.ts", "export {value} from '../src/a';");
    f.write("src/excluded.ts", "not valid syntax {{{");
    f.write("node_modules/package/a.ts", "not valid syntax {{{");
    f.move([["src/a.ts", "src/deep/a.ts"]]);
    expect(f.read("tools/use.ts")).toBe("export {value} from '../src/deep/a';");
    expect(f.read("src/excluded.ts")).toBe("not valid syntax {{{");
});

test("overlapping requests, cycles and collisions fail before any edit", () => {
    for (const pairs of [
        [["src/a.ts", "src/out/a.ts"], ["src/a.ts", "src/other/a.ts"]],
        [["src/a.ts", "src/b.ts"], ["src/b.ts", "src/a.ts"]],
        [["src", "src/nested"]],
        [["src/a.ts", "src/out/a.ts"], ["src/b.ts", "src/out/a.ts"]],
        [["src/a.ts", "src/out/a.ts"], ["src/missing.ts", "src/out/missing.ts"]],
    ] as [string, string][][]) {
        const f = fixture();
        f.write("src/a.ts", "export const a = 1;");
        f.write("src/b.ts", "export const b = 2;");
        const before = f.snapshot();
        expect(() => f.move(pairs)).toThrow();
        expect(f.snapshot()).toEqual(before);
    }
});
