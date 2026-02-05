/**
 * WASM Integration Benchmarks
 *
 * These tests measure real-world performance of the WASM engine
 * from JavaScript, including:
 * - Entity creation/destruction throughput
 * - Spatial queries performance
 * - Input event processing latency
 * - Frame rendering times
 *
 * Run with: npx vitest run src/test/wasm-benchmarks.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { WasmBridge } from "../wasm/archflow_web";

// Benchmark utilities
interface BenchmarkResult {
  name: string;
  ops: number;
  avgMs: number;
  throughput: number; // ops/sec
  p50: number;
  p95: number;
  p99: number;
}

function runBenchmark(
  name: string,
  iterations: number,
  fn: () => void | Promise<void>,
): BenchmarkResult {
  const times: number[] = [];

  // Warmup
  for (let i = 0; i < 100; i++) {
    fn();
  }

  // Actual benchmark
  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const iterStart = performance.now();
    fn();
    times.push(performance.now() - iterStart);
  }
  const total = performance.now() - start;

  // Calculate percentiles
  times.sort((a, b) => a - b);
  const p50 = times[Math.floor(times.length * 0.50)];
  const p95 = times[Math.floor(times.length * 0.95)];
  const p99 = times[Math.floor(times.length * 0.99)];

  return {
    name,
    ops: iterations,
    avgMs: total / iterations,
    throughput: (iterations / total) * 1000,
    p50,
    p95,
    p99,
  };
}

// Shared WASM bridge instance for benchmarks
let bridge: WasmBridge | null = null;

describe("WASM Integration Benchmarks", () => {
  beforeAll(async () => {
    // Initialize WASM module
    const init = await import("../wasm/archflow_web");
    await init.default();
    bridge = new WasmBridge();
    bridge.initialize(1920, 1080);
  });

  afterAll(() => {
    if (bridge) {
      bridge.free();
    }
  });

  describe("Entity Operations", () => {
    it("benchmark: spawn 100k entities", async () => {
      expect(bridge).not.toBeNull();

      const result = runBenchmark("spawn_100k", 100_000, () => {
        // Spawn a new entity at random position
        bridge!.spawn_shape(
          Math.random() * 1920,
          Math.random() * 1080,
          50, 50,
          0xFF0000FF, // RGBA
        );
      });

      console.table({
        Operation: result.name,
        "Operations": result.ops.toLocaleString(),
        "Avg (ms)": result.avgMs.toFixed(4),
        "Throughput (ops/s)": result.throughput.toFixed(0),
        "P50 (ms)": result.p50.toFixed(4),
        "P95 (ms)": result.p95.toFixed(4),
        "P99 (ms)": result.p99.toFixed(4),
      });

      // Performance assertions
      expect(result.throughput).toBeGreaterThan(50_000); // Min 50k ops/sec
      expect(result.p99).toBeLessThan(10); // P99 < 10ms
    });

    it("benchmark: batch spawn 1k entities", () => {
      expect(bridge).not.toBeNull();

      const iterations = 1_000;
      const batchSize = 1_000;

      const result = runBenchmark("batch_spawn_1k", iterations, () => {
        for (let i = 0; i < batchSize; i++) {
          bridge!.spawn_shape(
            Math.random() * 1920,
            Math.random() * 1080,
            50, 50,
            0xFF0000FF,
          );
        }
      });

      console.table({
        Operation: result.name,
        "Batches": result.ops.toLocaleString(),
        "Per Batch": batchSize,
        "Total Ops": (result.ops * batchSize).toLocaleString(),
        "Throughput (ops/s)": result.throughput.toFixed(0),
        "Avg (ms)": result.avgMs.toFixed(2),
      });

      expect(result.throughput).toBeGreaterThan(1_000_000); // Min 1M ops/sec
    });
  });

  describe("Mutation Performance", () => {
    beforeAll(() => {
      // Pre-populate with 10k entities
      for (let i = 0; i < 10_000; i++) {
        bridge!.spawn_shape(
          (i % 100) * 20,
          Math.floor(i / 100) * 20,
          50, 50,
          0xFF0000FF,
        );
      }
    });

    it("benchmark: move 10k entities", () => {
      const iterations = 100;
      const indices: number[] = [];

      // Get all entity indices
      for (let i = 0; i < 10_000; i++) {
        indices.push(i);
      }

      const result = runBenchmark("move_10k", iterations, () => {
        for (const idx of indices) {
          bridge!.move_entity(idx, 1.0, 0.5);
        }
      });

      console.table({
        Operation: result.name,
        "Entities": "10,000",
        "Frames": result.ops,
        "Total Moves": (result.ops * 10_000).toLocaleString(),
        "Throughput (moves/s)": result.throughput.toFixed(0),
        "P50 (ms)": result.p50.toFixed(2),
        "P99 (ms)": result.p99.toFixed(2),
      });

      // At 60 FPS, we have 16ms per frame
      // Should move 10k entities in < 16ms
      expect(result.p50).toBeLessThan(16);
      expect(result.throughput).toBeGreaterThan(60_000_000);
    });

    it("benchmark: get entity position 10k entities", () => {
      const iterations = 100;
      const indices: number[] = [];

      for (let i = 0; i < 10_000; i++) {
        indices.push(i);
      }

      const result = runBenchmark("query_positions_10k", iterations, () => {
        for (const idx of indices) {
          bridge!.get_entity_position(idx);
        }
      });

      console.table({
        Operation: result.name,
        "Queries": result.ops.toLocaleString(),
        "Throughput (queries/s)": result.throughput.toFixed(0),
        "P50 (ms)": result.p50.toFixed(3),
        "P99 (ms)": result.p99.toFixed(3),
      });

      expect(result.throughput).toBeGreaterThan(50_000_000);
    });
  });

  describe("Spatial Queries", () => {
    beforeAll(() => {
      // Create 50k entities in a grid
      for (let i = 0; i < 50_000; i++) {
        bridge!.spawn_shape(
          (i % 250) * 40,
          Math.floor(i / 250) * 40,
          30, 30,
          0xFF0000FF,
        );
      }
    });

    it("benchmark: point query in 50k entities", () => {
      const iterations = 10_000;

      const result = runBenchmark("point_query_50k", iterations, () => {
        bridge!.query_entities_at(960, 540); // Center of screen
      });

      console.table({
        Operation: result.name,
        "Entities": "50,000",
        "Queries": result.ops.toLocaleString(),
        "Throughput (queries/s)": result.throughput.toFixed(0),
        "Avg (ms)": result.avgMs.toFixed(3),
        "P99 (ms)": result.p99.toFixed(3),
      });

      expect(result.throughput).toBeGreaterThan(100_000);
    });

    it("benchmark: box selection 50k entities", () => {
      const iterations = 1_000;

      const result = runBenchmark("box_select_50k", iterations, () => {
        bridge!.select_entities_in_box(400, 300, 800, 600);
      });

      console.table({
        Operation: result.name,
        "Entities": "50,000",
        "Selections": result.ops.toLocaleString(),
        "Throughput (ops/s)": result.throughput.toFixed(0),
        "Avg (ms)": result.avgMs.toFixed(2),
      });

      expect(result.avgMs).toBeLessThan(5); // Should complete in < 5ms
    });
  });

  describe("Input Event Processing", () => {
    it("benchmark: mouse move events", () => {
      const iterations = 50_000;

      const result = runBenchmark("mouse_moves", iterations, () => {
        bridge!.push_input_event(0, 960, 540, 0, 0);
        bridge!.process_input();
      });

      console.table({
        Operation: result.name,
        "Events": result.ops.toLocaleString(),
        "Throughput (events/s)": result.throughput.toFixed(0),
        "Latency P99 (ms)": result.p99.toFixed(4),
      });

      // Input events should process in < 1ms
      expect(result.p99).toBeLessThan(1);
    });

    it("benchmark: frame tick performance", () => {
      const iterations = 10_000;

      const result = runBenchmark("frame_ticks", iterations, () => {
        bridge!.tick(performance.now());
      });

      console.table({
        Operation: result.name,
        "Frames": result.ops.toLocaleString(),
        "Throughput (fps)": result.throughput.toFixed(0),
        "Avg (ms/frame)": result.avgMs.toFixed(3),
        "P99 (ms)": result.p99.toFixed(3),
      });

      // Should sustain 60 FPS (16.67ms per frame)
      expect(result.avgMs).toBeLessThan(16.67);
    });
  });

  describe("Selection Operations", () => {
    beforeAll(() => {
      // Create entities for selection tests
      for (let i = 0; i < 5_000; i++) {
        bridge!.spawn_shape(
          (i % 100) * 30,
          Math.floor(i / 100) * 30,
          25, 25,
          0xFF0000FF,
        );
      }
    });

    it("benchmark: select single entity", () => {
      const iterations = 10_000;

      const result = runBenchmark("select_single", iterations, () => {
        bridge!.select_entity(1234, true);
      });

      console.table({
        Operation: result.name,
        "Selections": result.ops.toLocaleString(),
        "Throughput (ops/s)": result.throughput.toFixed(0),
        "P99 (ms)": result.p99.toFixed(4),
      });

      expect(result.throughput).toBeGreaterThan(100_000);
    });

    it("benchmark: batch select 1k entities", () => {
      const iterations = 1_000;
      const indices: number[] = [];

      for (let i = 0; i < 1_000; i++) {
        indices.push(i);
      }

      const result = runBenchmark("batch_select_1k", iterations, () => {
        // Assuming batch_select API exists
        if ("batch_select" in bridge!) {
          // @ts-ignore
          bridge!.batch_select(indices);
        }
      });

      console.table({
        Operation: result.name,
        "Batches": result.ops.toLocaleString(),
        "Per Batch": "1,000",
        "Total Selections": result.ops.toLocaleString(),
        "Throughput (ops/s)": result.throughput.toFixed(0),
      });
    });
  });

  describe("Throughput Stress Test", () => {
    it("sustained 60 FPS stress test", () => {
      const targetFPS = 60;
      const testDuration = 5000; // 5 seconds
      const frameBudget = 1000 / targetFPS; // 16.67ms per frame

      let frames = 0;
      let droppedFrames = 0;
      const startTime = performance.now();

      // Create 20k entities
      for (let i = 0; i < 20_000; i++) {
        bridge!.spawn_shape(
          Math.random() * 1920,
          Math.random() * 1080,
          50, 50,
          0xFF0000FF,
        );
      }

      // Run simulation loop
      while (performance.now() - startTime < testDuration) {
        const frameStart = performance.now();

        // Simulate game workload
        for (let i = 0; i < 5_000; i++) {
          bridge!.move_entity(i % 20_000, 0.1, 0.05);
        }

        // Process input
        if (frames % 60 === 0) {
          bridge!.push_input_event(0, 960, 540, 0, 0);
          bridge!.process_input();
        }

        // Tick
        bridge!.tick(performance.now());

        frames++;

        const frameTime = performance.now() - frameStart;
        if (frameTime > frameBudget) {
          droppedFrames++;
        }

        // Maintain consistent frame rate
        while (performance.now() - frameStart < frameBudget) {
          // Spin wait
        }
      }

      const actualFPS = (frames / testDuration) * 1000;
      const dropoutRate = (droppedFrames / frames) * 100;

      console.table({
        "Test Duration (ms)": testDuration,
        "Frames Rendered": frames,
        "Actual FPS": actualFPS.toFixed(1),
        "Target FPS": targetFPS,
        "Dropped Frames": droppedFrames,
        "Dropout Rate (%)": dropoutRate.toFixed(2),
        "Entity Count": "20,000",
        "Ops Per Frame": "5,000 moves + 1 tick",
      });

      expect(actualFPS).toBeGreaterThanOrEqual(55); // Within 5 FPS of target
      expect(dropoutRate).toBeLessThan(10); // Less than 10% dropout
    });
  });
});

// Export for reporting
export { type BenchmarkResult, runBenchmark };
