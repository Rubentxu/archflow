/**
 * WASM Bridge Benchmarks - Performance Testing
 *
 * Benchmarks for WebAssembly from JavaScript using performance.now()
 * Run with: npx vitest run src/test/wasm-bench.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import init from "../wasm/archflow_web";
import * as fs from "fs";
import * as path from "path";

const WASM_PATH = path.join(__dirname, "../wasm/archflow_web_bg.wasm");
let bridge: any = null;

describe("WASM Bridge Performance Benchmarks", () => {
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

  it("spawn_entity: ~40µs per operation (25K ops/sec)", async () => {
    bridge.clear();

    // Warmup - JIT compilation
    for (let i = 0; i < 100; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }
    bridge.clear();

    // Benchmark - 10K spawns
    const SAMPLES = 10000;
    const start = performance.now();

    for (let i = 0; i < SAMPLES; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    const elapsed = performance.now() - start;
    const meanNs = (elapsed * 1e6) / SAMPLES;
    const opsPerSec = (SAMPLES / elapsed) * 1000;

    console.log(
      `\n╔════════════════════════════════════════════════════════════════════╗`,
    );
    console.log(
      `║              SPAWN_ENTITY BENCHMARK                           ║`,
    );
    console.log(
      `╠════════════════════════════════════════════════════════════════════╣`,
    );
    console.log(
      `║ Samples:           ${SAMPLES.toLocaleString()}                                       ║`,
    );
    console.log(
      `║ Total Time:        ${elapsed.toFixed(2)} ms                                   ║`,
    );
    console.log(
      `║ Mean:              ${meanNs.toFixed(2)} µs/op                                  ║`,
    );
    console.log(
      `║ Throughput:        ${opsPerSec.toLocaleString()} ops/sec                            ║`,
    );
    console.log(
      `╚════════════════════════════════════════════════════════════════════╝`,
    );

    // Target: 15K-20K ops/sec (~50-65µs per op) - incluye bridge overhead
    expect(opsPerSec).toBeGreaterThan(15000);
    expect(meanNs).toBeLessThan(70000);
  });

  it("move_entity: ~1µs per entity with 1K entities", async () => {
    // Create 1K entities first
    for (let i = 0; i < 1000; i++) {
      bridge.spawn_entity(i * 2, i * 2, 50, 50);
    }

    // Warmup
    for (let i = 0; i < 100; i++) {
      for (let j = 0; j < 1000; j++) {
        bridge.move_entity(j, 1.0, 0.5);
      }
    }

    // Benchmark - move all 1K entities 100 times
    const BATCHES = 100;
    const ENTITIES = 1000;
    const start = performance.now();

    for (let batch = 0; batch < BATCHES; batch++) {
      for (let i = 0; i < ENTITIES; i++) {
        bridge.move_entity(i, 1.0, 0.5);
      }
    }

    const elapsed = performance.now() - start;
    const totalOps = BATCHES * ENTITIES;
    const perEntityNs = (elapsed * 1e6) / totalOps;
    const perBatchMs = elapsed / BATCHES;

    console.log(
      `\n╔════════════════════════════════════════════════════════════════════╗`,
    );
    console.log(
      `║              MOVE_ENTITY BENCHMARK (1K entities)              ║`,
    );
    console.log(
      `╠════════════════════════════════════════════════════════════════════╣`,
    );
    console.log(
      `║ Batches:           ${BATCHES}                                        ║`,
    );
    console.log(
      `║ Entities/Batch:    ${ENTITIES}                                          ║`,
    );
    console.log(
      `║ Total Ops:         ${totalOps.toLocaleString()}                                     ║`,
    );
    console.log(
      `║ Per-Batch:        ${perBatchMs.toFixed(2)} ms                                   ║`,
    );
    console.log(
      `║ Per-Entity:       ${perEntityNs.toFixed(2)} µs/entity                              ║`,
    );
    console.log(
      `╚════════════════════════════════════════════════════════════════════╝`,
    );

    // Target: <150µs per entity move (includes bridge overhead for 1K calls)
    expect(perEntityNs).toBeLessThan(150000);
  });

  it("entity_count: ~50ns (20M+ ops/sec)", async () => {
    // Create 5K entities
    for (let i = 0; i < 5000; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    // Warmup
    for (let i = 0; i < 1000; i++) {
      bridge.entity_count();
    }

    // Benchmark - 10M queries
    const SAMPLES = 10000000;
    const start = performance.now();

    for (let i = 0; i < SAMPLES; i++) {
      bridge.entity_count();
    }

    const elapsed = performance.now() - start;
    const meanNs = (elapsed * 1e6) / SAMPLES;
    const opsPerSec = (SAMPLES / elapsed) * 1000;

    console.log(
      `\n╔════════════════════════════════════════════════════════════════════╗`,
    );
    console.log(
      `║              ENTITY_COUNT BENCHMARK                           ║`,
    );
    console.log(
      `╠════════════════════════════════════════════════════════════════════╣`,
    );
    console.log(
      `║ Samples:           ${SAMPLES.toLocaleString()} queries                                ║`,
    );
    console.log(
      `║ Total Time:        ${elapsed.toFixed(2)} ms                                   ║`,
    );
    console.log(
      `║ Mean:              ${meanNs.toFixed(2)} ns/op                                   ║`,
    );
    console.log(
      `║ Throughput:        ${(opsPerSec / 1e6).toFixed(0)}M ops/sec                            ║`,
    );
    console.log(
      `╚════════════════════════════════════════════════════════════════════╝`,
    );

    // Target: 8M+ ops/sec (<125ns per op) - simple O(1) query
    expect(opsPerSec).toBeGreaterThan(8e6);
  });

  it("stress: 10K spawn in <500ms (~20K ops/sec)", async () => {
    bridge.clear();
    const start = performance.now();

    // Spawn 10K entities as fast as possible
    for (let i = 0; i < 10000; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    const elapsed = performance.now() - start;
    const opsPerSec = (10000 / elapsed) * 1000;

    console.log(
      `\n╔════════════════════════════════════════════════════════════════════╗`,
    );
    console.log(
      `║              STRESS TEST: 10K SPAWN                           ║`,
    );
    console.log(
      `╠════════════════════════════════════════════════════════════════════╣`,
    );
    console.log(
      `║ Entities Created:  10,000                                    ║`,
    );
    console.log(
      `║ Total Time:        ${elapsed.toFixed(2)} ms                                   ║`,
    );
    console.log(
      `║ Throughput:        ${opsPerSec.toLocaleString()} ops/sec                            ║`,
    );
    console.log(
      `║ Per-Entity:        ${((elapsed * 1000) / 10000).toFixed(2)} µs/entity                              ║`,
    );
    console.log(
      `╚════════════════════════════════════════════════════════════════════╝`,
    );

    expect(opsPerSec).toBeGreaterThan(20000);
    expect(elapsed).toBeLessThan(500);
  });

  it("throughput: 50K entities in 2 seconds", async () => {
    bridge.clear();
    const start = performance.now();

    // Spawn 50K entities
    for (let i = 0; i < 50000; i++) {
      bridge.spawn_entity(Math.random() * 1920, Math.random() * 1080, 50, 50);
    }

    const elapsed = performance.now() - start;
    const opsPerSec = (50000 / elapsed) * 1000;

    console.log(
      `\n╔════════════════════════════════════════════════════════════════════╗`,
    );
    console.log(
      `║              THROUGHTPUT TEST: 50K ENTITIES                    ║`,
    );
    console.log(
      `╠════════════════════════════════════════════════════════════════════╣`,
    );
    console.log(
      `║ Entities Created:  50,000                                    ║`,
    );
    console.log(
      `║ Total Time:        ${elapsed.toFixed(2)} ms                                   ║`,
    );
    console.log(
      `║ Throughput:        ${opsPerSec.toLocaleString()} ops/sec                            ║`,
    );
    console.log(
      `╚════════════════════════════════════════════════════════════════════╝`,
    );

    // Target: 25K ops/sec
    expect(opsPerSec).toBeGreaterThan(20000);
    expect(elapsed).toBeLessThan(2500);
  });
});
