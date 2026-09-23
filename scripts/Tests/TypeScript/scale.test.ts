import { expect, test } from "bun:test";
import { fixture } from "./fixture";

test("five-file cyclic batch updates 3000 outside callers with exact output", () => {
    const f = fixture();
    const pairs: [string, string][] = [];
    for (let i = 0; i < 5; i++) {
        f.write(`src/modules/m${i}.ts`, `export {v${(i + 1) % 5}} from './m${(i + 1) % 5}'; export const v${i} = ${i};\n`);
        pairs.push([`src/modules/m${i}.ts`, `src/target${i}/m${i}.ts`]);
    }
    for (let i = 0; i < 3000; i++) {
        f.write(`src/callers/c${i}.ts`, `export {v${i % 5}} from '../modules/m${i % 5}'; // caller ${i}\n`);
    }
    const plan = f.move(pairs);
    expect(plan.changes).toHaveLength(3005);
    for (let i = 0; i < 3000; i++) {
        expect(f.read(`src/callers/c${i}.ts`)).toBe(`export {v${i % 5}} from '../target${i % 5}/m${i % 5}'; // caller ${i}\n`);
    }
    for (let i = 0; i < 5; i++) {
        expect(f.read(`src/target${i}/m${i}.ts`)).toContain(`'../target${(i + 1) % 5}/m${(i + 1) % 5}'`);
    }
});
