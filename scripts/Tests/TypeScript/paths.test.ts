import { expect, test } from "bun:test";
import { fixture } from "./fixture";

test("inherited tsconfig aliases update by resolved target, including prefix-overlapping destinations", () => {
    const f = fixture({ extends: "./config/base.json", include: ["src/**/*"] });
    f.write("config/base.json", JSON.stringify({ compilerOptions: { baseUrl: "..", moduleResolution: "bundler", module: "esnext", paths: { "~/*": ["src/*"], "@fixed": ["src/Entry.ts"] } } }));
    f.write("src/Entry.ts", "export const value = 1;");
    f.write("src/Entry_Extra.ts", "export const other = 2;");
    f.write("src/caller.ts", `import {value} from '~/Entry';\nexport {value as explicit} from '~/Entry.ts';\nexport {value as fixed} from '@fixed';\nexport {other} from '~/Entry_Extra';\n`);
    f.move([["src/Entry.ts", "src/Entry/Page.ts"]]);
    expect(f.read("src/caller.ts")).toBe(`import {value} from '~/Entry/Page';\nexport {value as explicit} from '~/Entry/Page.ts';\nexport {value as fixed} from './Entry/Page';\nexport {other} from '~/Entry_Extra';\n`);
});

test("directory moves carry assets and preserve query and fragment suffixes", () => {
    const f = fixture();
    f.write("src/ui/view.ts", `import './style.css';\nimport icon from './icon.svg?raw#logo';\nexport {value} from '../value';\n`);
    f.write("src/ui/style.css", "/* unchanged */\n");
    f.write("src/ui/icon.svg", "<svg/>\n");
    f.write("src/value.ts", "export const value = 1;");
    f.write("src/caller.ts", `import './ui/style.css?inline';\nimport './ui/view';\n`);
    f.move([["src/ui", "src/deep/ui"]]);
    expect(f.read("src/deep/ui/view.ts")).toContain("from '../../value'");
    expect(f.read("src/deep/ui/view.ts")).toContain("'./icon.svg?raw#logo'");
    expect(f.read("src/caller.ts")).toBe(`import './deep/ui/style.css?inline';\nimport './deep/ui/view';\n`);
    expect(f.read("src/deep/ui/icon.svg")).toBe("<svg/>\n");
    expect(f.exists("src/ui")).toBe(false);
});

test("ESM .js specifiers continue pointing to their .ts source", () => {
    const f = fixture({ compilerOptions: { module: "nodenext", moduleResolution: "nodenext" }, include: ["src/**/*"] });
    f.write("package.json", '{"type":"module"}');
    f.write("src/a.ts", "export const value = 1;");
    f.write("src/caller.ts", "export {value} from './a.js';");
    f.move([["src/a.ts", "src/core/b.ts"]]);
    expect(f.read("src/caller.ts")).toBe("export {value} from './core/b.js';");
});

test("classic resolution and rootDirs retain compiler semantics", () => {
    const f = fixture({ compilerOptions: { module: "esnext", rootDirs: ["src", "generated"] }, include: ["src/**/*", "generated/**/*"] });
    f.write("src/a.ts", "export {value} from './b';");
    f.write("generated/b.ts", "export const value = 1;");
    f.move([["src/a.ts", "src/deep/a.ts"]]);
    expect(f.read("src/deep/a.ts")).toContain("../../generated/b");
});

test("type-only resolution-mode and dynamic imports use the correct package export", () => {
    const f = fixture({ compilerOptions: { module: "nodenext", moduleResolution: "nodenext" }, include: ["src/**/*"] });
    f.write("package.json", JSON.stringify({ name: "self", type: "commonjs", exports: { ".": { import: "./src/esm.ts", require: "./src/cjs.ts" } } }));
    f.write("src/esm.ts", "export const esm = 1; export type Kind = string;");
    f.write("src/cjs.ts", "export const cjs = 1; export type Kind = number;");
    f.write("src/caller.ts", `import type {Kind} from 'self' with { 'resolution-mode': 'import' };\ntype K = import('self', {with: {'resolution-mode': 'import'}}).Kind;\nconst promise = import('self');\nconst common = require('self');\n`);
    f.move([["src/esm.ts", "src/deep/esm.ts"]]);
    const caller = f.read("src/caller.ts");
    expect(caller.match(/\.\/deep\/esm/g)?.length).toBe(3);
    expect(caller).toContain("require('self')");
});

test("moduleSuffixes preserve the compiler-selected platform variant", () => {
    const f = fixture({ compilerOptions: { moduleResolution: "bundler", module: "esnext", moduleSuffixes: [".native"] }, include: ["src/**/*"] });
    f.write("src/a.native.ts", "export const value = 1;");
    f.write("src/caller.ts", "export {value} from './a';");
    f.move([["src/a.native.ts", "src/deep/b.native.ts"]]);
    expect(f.read("src/caller.ts")).toBe("export {value} from './deep/b';");
});

test("directory batches preserve mts/cts emitted extensions and JSON attributes", () => {
    const f = fixture({ compilerOptions: { moduleResolution: "nodenext", module: "nodenext", resolveJsonModule: true }, include: ["src/**/*"] });
    f.write("package.json", '{"type":"module"}');
    f.write("src/modules/a.mts", "export const value = 1;");
    f.write("src/modules/b.cts", "export const other = 2;");
    f.write("src/modules/data.json", '{"count":3}');
    f.write("src/caller.ts", `export {value} from './modules/a.mjs';\nexport {other} from './modules/b.cjs';\nimport data from './modules/data.json' with { type: 'json' };\n`);
    f.move([["src/modules", "src/deep/modules"]]);
    expect(f.read("src/caller.ts")).toBe(`export {value} from './deep/modules/a.mjs';\nexport {other} from './deep/modules/b.cjs';\nimport data from './deep/modules/data.json' with { type: 'json' };\n`);
});
