import { expect, test } from "bun:test";
import { prepareMoves } from "../../TypeScript/moves";
import { planMoves, verifyPlan } from "../../TypeScript/plan";
import { readProject } from "../../TypeScript/project";
import { afterMoves } from "../../TypeScript/virtual";
import { fixture } from "./fixture";

type Fixture = ReturnType<typeof fixture>;

/** What `--dry-run` does: plan, then check against the files as they will be. */
function dryRun(f: Fixture, pairs: [string, string][]): number {
    const moves = prepareMoves(f.root, pairs);
    const project = readProject(f.root);
    return verifyPlan(project, planMoves(project, moves), afterMoves(moves));
}

/** The dry run and the real move must both refuse or both succeed, and the dry run writes nothing. */
function agree(f: Fixture, pairs: [string, string][], refusal?: string) {
    const before = f.snapshot();
    let dry: unknown;
    try { dryRun(f, pairs); } catch (error) { dry = error; }
    expect(f.snapshot()).toEqual(before);
    let real: unknown;
    try { f.move(pairs); } catch (error) { real = error; }
    if (refusal === undefined) {
        expect(dry).toBeUndefined();
        expect(real).toBeUndefined();
        return;
    }
    expect(String(dry)).toContain(refusal);
    expect(String(real)).toContain("original files restored");
}

test("a dry run refuses a move whose new place another file answers", () => {
    const f = fixture();
    f.write("src/a.tsx", "export const value = 1;");
    f.write("src/caller.ts", "export {value} from './a';");
    f.write("src/new/a.ts", "export const wrong = 2;");
    agree(f, [["src/a.tsx", "src/new/a.tsx"]], "Moved module resolves incorrectly");
});

test("a dry run refuses a file that a folder of the same name would not beat", () => {
    const f = fixture({ compilerOptions: { module: "commonjs", moduleResolution: "node" }, include: ["src/**/*"] });
    f.write("src/old.ts", "export const old = 1;");
    f.write("src/lib.ts", "export const lib = 2;");
    f.write("src/main.ts", "import { old } from './old';\nimport { lib } from './lib';\nconsole.log(old, lib);");
    agree(f, [["src/old.ts", "src/lib/index.ts"]], "Moved module resolves incorrectly");
});

test("a dry run accepts what the real move accepts, folders and aliases included", () => {
    const f = fixture({
        compilerOptions: { module: "esnext", moduleResolution: "bundler", baseUrl: ".", paths: { "@lib/*": ["src/lib/*"] } },
        include: ["src/**/*"],
    });
    f.write("src/lib/util.ts", "export const util = 1;");
    f.write("src/lib/extra/more.ts", "import { util } from '../util';\nexport const more = util + 1;");
    f.write("src/app.ts", "import { util } from '@lib/util';\nimport { more } from './lib/extra/more';\nconsole.log(util, more);");
    agree(f, [["src/lib", "src/shared/lib"]]);
});

test("a dry run leaves alone the alias imports of assets that do not move", () => {
    const f = fixture({ compilerOptions: { module: "esnext", moduleResolution: "bundler", baseUrl: ".", paths: { "@assets/*": ["src/assets/*"] } }, include: ["src/**/*"] });
    f.write("src/assets/logo.svg", "<svg/>");
    f.write("src/app.ts", "import logo from '@assets/logo.svg';\nconsole.log(logo);");
    expect(dryRun(f, [["src/app.ts", "src/pages/app.ts"]])).toBe(0);
});

test("a dry run counts the rewritten alias imports of assets it cannot check", () => {
    const f = fixture({ compilerOptions: { module: "esnext", moduleResolution: "bundler", baseUrl: ".", paths: { "@assets/*": ["src/assets/*"] } }, include: ["src/**/*"] });
    f.write("src/assets/logo.svg", "<svg/>");
    f.write("src/app.ts", "import logo from '@assets/logo.svg';\nconsole.log(logo);");
    const pairs: [string, string][] = [["src/assets/logo.svg", "src/assets/img/logo.svg"]];
    expect(dryRun(f, pairs)).toBe(1);
    // The real move checks the import against the disk and accepts it.
    f.move(pairs);
    expect(f.read("src/app.ts")).toContain("@assets/img/logo.svg");
});

test("a dry run checks the relative asset imports it rewrites", () => {
    const f = fixture();
    f.write("src/assets/logo.svg", "<svg/>");
    f.write("src/app.ts", "import logo from './assets/logo.svg';\nconsole.log(logo);");
    agree(f, [["src/assets/logo.svg", "src/assets/img/logo.svg"]]);
});
