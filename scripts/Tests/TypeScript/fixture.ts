import { afterEach } from "bun:test";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { applyPlan } from "../../TypeScript/apply";
import { prepareMoves } from "../../TypeScript/moves";
import { planMoves } from "../../TypeScript/plan";
import { readProject } from "../../TypeScript/project";

const roots: string[] = [];
afterEach(() => { for (const root of roots.splice(0)) fs.rmSync(root, { recursive: true }); });

export function fixture(config: unknown = { compilerOptions: { module: "esnext", moduleResolution: "bundler", allowJs: true }, include: ["src/**/*"] }) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "refac-oxc-test-"));
    roots.push(root);
    function write(file: string, text: string | Buffer) {
        fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
        fs.writeFileSync(path.join(root, file), text);
    }
    write("tsconfig.json", JSON.stringify(config));
    return {
        root, write,
        read(file: string) { return fs.readFileSync(path.join(root, file), "utf8"); },
        exists(file: string) { return fs.existsSync(path.join(root, file)); },
        move(pairs: [string, string][]) {
            const moves = prepareMoves(root, pairs);
            const project = readProject(root);
            const plan = planMoves(project, moves);
            applyPlan(project, plan);
            return plan;
        },
        snapshot() {
            const files: Record<string, string> = {};
            function visit(directory: string) {
                for (const name of fs.readdirSync(directory).sort()) {
                    const file = path.join(directory, name), stat = fs.lstatSync(file);
                    if (stat.isSymbolicLink()) files[path.relative(root, file)] = `symlink:${fs.readlinkSync(file)}`;
                    else if (stat.isDirectory()) visit(file);
                    else files[path.relative(root, file)] = fs.readFileSync(file).toString("base64");
                }
            }
            visit(root);
            return files;
        },
    };
}
