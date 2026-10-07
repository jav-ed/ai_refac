import { total, computeSum as sum, Counter, type Options, Color } from "./lib/util";
import * as util from "./lib/util";
import defaultFn from "./lib/util";
export { total } from "./lib/util";

const local = total + 1;
const shorthand = { total };
const c = new Counter();
c.increment();
console.log(c.count, sum([1, 2]), util.total, util.computeSum([3]), Color.Red, defaultFn());
const opts: Options = { verbose: true };
function shadow(total: number) { return total * 2; } // unrelated param named total
const str = "total in string"; // total in comment
export { local, shorthand, opts, shadow, str };
