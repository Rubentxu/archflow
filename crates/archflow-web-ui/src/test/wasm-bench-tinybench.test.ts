/**
 * WASM Bridge Benchmarks using Tinybench
 *
 * Proper benchmarking for WebAssembly from JavaScript
 * Run with: npx vitest run src/test/wasm-bench-tinybench.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Bench } from "tinybench";
import init from "../wasm/archflow_web";
import * as fs from "fs";
import * as path from "path";

const WASM_PATH = path.join(__dirname, "../wasm/archflow_web_bg.wasm");
let bridge: any = null;

describe("WASM Bridge Benchmarks (Tinybench)", () => {
  beforeAll(async () => {
    // Load WASM buffer directly for Node.js environment
    const wasmBuffer = fs.readFileSync(WASM_PATH);
    const wasmModule = new WebAssembly.Module(wasmBuffer);
    await init({ module_or_path: wasmModule });
    bridge = new (await import("../wasm/archflow_web")).WasmBridge();
    bridge.initialize(1920, 1080);
  });

  afterAll(() => {
    if (bridge) bridge.free();
  });

  it("benchmark: spawn_entity throughput", async () => {
    bridge.clear();

    const bench = new Bench({
      name: "spawn_entity",
      time: 500, // Run for 500ms
      warmup: 50, // 50 warmup iterations
    });

    const spawnFn = () => {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    };

    // Warmup phase
    for (let i = 0; i < 50; i++) {
      spawnFn();
    }
    bridge.clear();

    // Benchmark phase
    bench.add("spawn_entity", spawnFn);
    await bench.run();

    // Tinybench v0.7+ API: results is a Map
    const result = bench.results.get("spawn_entity");

    if (!result) {
      throw new Error("Benchmark result not found");
    }

    const meanNs = result.mean * 1e6; // Convert to nanoseconds
    const opsPerSec = 1e9 / meanNs;

    console.log(
      "\n╔════════════════════════════════════════════════════════════════════╗",
    );
    console.log(
      "║              SPAWN ENTITY BENCHMARK RESULTS                     ║",
    );
    console.log(
      "╠════════════════════════════════════════════════════════════════════╣",
    );
    console.log(
      `║ Samples:           ${result.samples.length} iterations                             ║`,
    );
    console.log(
      `║ Mean:              ${meanNs.toFixed(2)} ns/op (${opsPerSec.toLocaleString()} ops/sec)        ║`,
    );
    console.log(
      `║ Std Dev:          ${(result.sd * 1e6).toFixed(2)} ns                                    ║`,
    );
    console.log(
      `║ Min:               ${(result.min * 1e6).toFixed(2)} ns                                    ║`,
    );
    console.log(
      `║ Max:               ${(result.max * 1e6).toFixed(2)} ns                                    ║`,
    );
    console.log(
      "╚════════════════════════════════════════════════════════════════════╝",
    );

    // Basic assertions - performance targets
    expect(opsPerSec).toBeGreaterThan(10000); // At least 10K ops/sec
    expect(meanNs).toBeLessThan(100000); // Less than 100µs per op
  });

  it("benchmark: move_entity throughput", async () => {
    // Create 1000 entities first
    for (let i = 0; i < 1000; i++) {
      bridge.spawn_entity(i * 2, i * 2, 50, 50);
    }

    const bench = new Bench({
      name: "move_entity_1k",
      time: 500,
      warmup: 20,
    });

    const moveFn = () => {
      for (let i = 0; i < 1000; i++) {
        bridge.move_entity(i, 1.0, 0.5);
      }
    };

    // Warmup
    for (let i = 0; i < 20; i++) {
      moveFn();
    }

    // Benchmark
    bench.add("move_entity_1k", moveFn);
    await bench.run();

    const result = bench.results.get("move_entity_1k");
    if (!result) {
      throw new Error("Benchmark result not found");
    }

    const perEntityNs = (result.mean * 1e6) / 1000;

    console.log(
      "\n╔════════════════════════════════════════════════════════════════════╗",
    );
    console.log(
      "║              MOVE ENTITY BENCHMARK (1K entities)                ║",
    );
    console.log(
      "╠════════════════════════════════════════════════════════════════════╣",
    );
    console.log(
      `║ Samples:           ${result.samples.length} batches                              ║`,
    );
    console.log(
      `║ Batch Time:        ${(result.mean * 1e6).toFixed(2)} ns/batch                          ║`,
    );
    console.log(
      `║ Per-Entity:       ${perEntityNs.toFixed(2)} ns/entity                           ║`,
    );
    console.log(
      `║ Throughput:        ${(1e9 / perEntityNs).toLocaleString()} ops/sec                       ║`,
    );
    console.log(
      "╚════════════════════════════════════════════════════════════════════╝",
    );

    expect(perEntityNs).toBeLessThan(5000); // Less than 5µs per entity move
  });

  it("benchmark: entity_count query", async () => {
    // Create some entities
    for (let i = 0; i < 5000; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    const bench = new Bench({
      name: "entity_count",
      time: 500,
      warmup: 50,
    });

    const countFn = () => {
      bridge.entity_count();
    };

    // Warmup
    for (let i = 0; i < 50; i++) {
      countFn();
    }

    bench.add("entity_count", countFn);
    await bench.run();

    const result = bench.results.get("entity_count");
    if (!result) {
      throw new Error("Benchmark result not found");
    }

    const meanNs = result.mean * 1e6;

    console.log(
      "\n╔════════════════════════════════════════════════════════════════════╗",
    );
    console.log(
      "║              ENTITY_COUNT QUERY BENCHMARK                     ║",
    );
    console.log(
      "╠════════════════════════════════════════════════════════════════════╣",
    );
    console.log(
      `║ Mean:              ${meanNs.toFixed(2)} ns/op                              ║`,
    );
    console.log(
      `║ Throughput:        ${(1e9 / meanNs).toLocaleString()} ops/sec                          ║`,
    );
    console.log(
      `║ Total Entities:    5,000                                    ║`,
    );
    console.log(
      "╚════════════════════════════════════════════════════════════════════╝",
    );

    // Should be very fast - O(1) operation
    expect(meanNs).toBeLessThan(10000); // Less than 10µs
  });

  it("stress test: 10K spawn in single frame", async () => {
    bridge.clear();
    const start = performance.now();

    // Spawn 10K entities as fast as possible
    for (let i = 0; i < 10000; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    const elapsed = performance.now() - start;
    const opsPerSec = (10000 / elapsed) * 1000;

    console.log(
      "\n╔════════════════════════════════════════════════════════════════════╗",
    );
    console.log(
      "║              STRESS TEST: 10K SPAWN                            ║",
    );
    console.log(
      "╠════════════════════════════════════════════════════════════════════╣",
    );
    console.log(
      `║ Entities Created:  10,000                                    ║`,
    );
    console.log(
      `║ Total Time:        ${elapsed.toFixed(2)} ms                              ║`,
    );
    console.log(
      `║ Throughput:          ${opsPerSec.toLocaleString()} ops/sec                        ║`,
    );
    console.log(
      `║ Per-Entity:         ${((elapsed * 1000) / 10000).toFixed(2)} µs/entity                           ║`,
    );
    console.log(
      "╚════════════════════════════════════════════════════════════════════╝",
    );

    expect(opsPerSec).toBeGreaterThan(20000); // At least 20K ops/sec
    expect(elapsed).toBeLessThan(500); // Should complete in under 500ms
  });
});
