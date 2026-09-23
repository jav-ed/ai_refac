import { expect, test } from "bun:test";
import { fixture } from "./fixture";

test("all literal module forms update while comments and ordinary strings stay byte-identical", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1; export type Kind = string;");
    const forms = [
        `import './a';`, `import {value} from "./a";`, `import type {Kind} from './a';`,
        `export {value as other} from './a';`, `export type {Kind as OtherKind} from './a';`,
        `export * from './a';`, `export * as ns from './a';`,
        `const dynamic = import('./a');`, "const template = import(`./a`);",
        `const cjs = require('./a');`, `import equal = require('./a');`,
        `type TypeImport = import('./a').Kind;`, `type TypeofImport = typeof import('./a');`,
    ];
    const untouched = `// Keep './a' here\nconst text = './a';\nconst expression = import('./' + text);\n`;
    f.write("src/caller.ts", forms.join("\n") + "\n" + untouched);
    f.move([["src/a.ts", "src/deep/b.ts"]]);
    expect(f.read("src/caller.ts")).toBe(forms.join("\n").replaceAll("./a", "./deep/b") + "\n" + untouched);
});

test("module augmentations follow their resolved source without touching ambient names", () => {
    const f = fixture();
    f.write("src/a.ts", "export interface Value {}");
    f.write("src/augment.d.ts", "export {};\ndeclare module './a' { interface Value { x: number } }\ndeclare module 'virtual:example' {}\n");
    f.move([["src/a.ts", "src/deep/a.ts"]]);
    expect(f.read("src/augment.d.ts")).toBe("export {};\ndeclare module './deep/a' { interface Value { x: number } }\ndeclare module 'virtual:example' {}\n");
});

test("Unicode, BOM, CRLF, escaped paths and original quote style survive precise edits", () => {
    const f = fixture();
    f.write("src/a.ts", "export const value = 1;");
    const source = '\uFEFF// 🌿 اردو Ä\r\nconst emoji = "😀";\r\nimport {value} from "./\\u0061";\r\n';
    f.write("src/caller.ts", source);
    f.move([["src/a.ts", "src/it's 🌿/b.ts"]]);
    expect(f.read("src/caller.ts")).toBe(source.replace('"./\\u0061"', '"./it\'s 🌿/b"'));
});

test("outgoing imports in a moved file update together with incoming callers", () => {
    const f = fixture();
    f.write("src/a.ts", "export {value} from './b';\n");
    f.write("src/b.ts", "export const value = 1;\n");
    f.write("src/caller.ts", "export * from './a';\n");
    f.move([["src/a.ts", "src/deeper/a.ts"]]);
    expect(f.read("src/deeper/a.ts")).toBe("export {value} from '../b';\n");
    expect(f.read("src/caller.ts")).toBe("export * from './deeper/a';\n");
});
