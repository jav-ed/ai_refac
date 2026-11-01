import { afterEach, expect, test } from "bun:test";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { applyPlan } from "../../TypeScript/apply";
import { prepareMoves } from "../../TypeScript/moves";
import { planMoves } from "../../TypeScript/plan";
import { readProject } from "../../TypeScript/project";

const roots: string[] = [];
afterEach(() => { for (const root of roots.splice(0)) fs.rmSync(root, { recursive: true }); });

// A project without a tsconfig.json, with a vendored tree that `.gitignore` lists.
function plainProject({ git }: { git: boolean }) {
    const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "refac-oxc-ignored-")));
    roots.push(root);
    function write(file: string, text: string) {
        fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
        fs.writeFileSync(path.join(root, file), text);
    }
    if (git) Bun.spawnSync(["git", "init", "--quiet"], { cwd: root });
    write(".gitignore", "vendor/\n");
    write("src/a.ts", "export const value = 1;\n");
    write("src/b.ts", "export { value } from './a';\n");
    return {
        root, write,
        files: () => readProject(root).files.map(file => path.relative(root, file)).sort(),
        move(from: string, to: string) {
            const moves = prepareMoves(root, [[from, to]], []);
            const project = readProject(root);
            applyPlan(project, planMoves(project, moves));
        },
    };
}

// Rule: without a tsconfig.json the project is the files git does not ignore: tracked ones and
// untracked ones that are not ignored (a file created a minute ago). A vendored tree that
// `.gitignore` lists is never scanned. Benefit 9/10: a Markdown move in a repo that vendors other
// projects started the engine over all of them and failed on a file nobody owns.
test("without a tsconfig.json the ignored trees are left out and a new file is kept", () => {
    const p = plainProject({ git: true });
    p.write("vendor/pkg/lib.ts", "export const lib = 1;\n");
    p.write("src/new.ts", "export const fresh = 1;\n");
    expect(p.files()).toEqual(["src/a.ts", "src/b.ts", "src/new.ts"]);
});

// Rule: outside a git work tree nothing says what is ignored, so TypeScript's own default
// stands (every file under the root). Benefit 5/10: keeps a plain folder working as before.
test("outside a git work tree the whole folder is read", () => {
    const p = plainProject({ git: false });
    p.write("vendor/pkg/lib.ts", "export const lib = 1;\n");
    expect(p.files()).toEqual(["src/a.ts", "src/b.ts", "vendor/pkg/lib.ts"]);
});

// Rule: a file the engine cannot parse is a hard error with its name when the project owns it,
// and is never read when the project ignores it. No silent skip of a failing file.
// Benefit 9/10: this is the line between "ignored" and "swallowed".
test("an unparsable ignored file is never read, an unparsable project file is a named error", () => {
    const p = plainProject({ git: true });
    p.write("vendor/pkg/broken.ts", "export const = ;\n");
    p.move("src/a.ts", "src/deep/a.ts");
    expect(fs.readFileSync(path.join(p.root, "src/b.ts"), "utf8")).toContain("./deep/a");

    p.write("src/broken.ts", "export const = ;\n");
    expect(() => p.move("src/deep/a.ts", "src/a.ts")).toThrow(/Cannot parse .*src\/broken\.ts/);
});
