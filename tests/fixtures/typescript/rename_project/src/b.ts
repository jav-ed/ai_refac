export * from "./lib/util";
export * as ns from "./lib/util";
const dyn = async () => { const { total } = await import("./lib/util"); return total; };
export { dyn };
