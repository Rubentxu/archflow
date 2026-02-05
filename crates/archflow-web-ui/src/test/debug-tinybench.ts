/**
 * WASM Bridge Benchmarks using Tinybench
 *
 * Debug script to check Tinybench API
 */

import { Bench } from "tinybench";

const bench = new Bench({ name: "test", time: 100 });

bench.add("test", () => {
  // empty
});
await bench.run();

console.log("bench.results type:", typeof bench.results);
console.log("bench.results:", JSON.stringify(bench.results, null, 2));
console.log("bench.tasks:", bench.tasks);
console.log("bench.taskNames:", bench.taskNames);
