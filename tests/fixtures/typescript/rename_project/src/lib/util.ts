export const total = 10;
export function computeSum(items: number[]): number {
    const total = items.reduce((a, b) => a + b, 0); // shadows module-level name only if exported total were here
    return total;
}
export class Counter {
    count = 0;
    increment(): void { this.count++; }
}
export interface Options { verbose: boolean }
export type Mode = "a" | "b";
export enum Color { Red, Green }
export default function defaultFn() { return 1; }
