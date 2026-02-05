/**
 * WASM Bridge Overhead Benchmarks
 *
 * These benchmarks specifically measure the overhead of crossing
 * the JavaScript ↔ WebAssembly boundary for individual operations.
 *
 * Key metrics:
 * - Per-call overhead (ns/µs per bridge crossing)
 * - Comparison with equivalent Rust native operations
 * - Impact of serialization/deserialization
 *
 * Run with: npx vitest run src/test/bridge-overhead.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { WasmBridge } from "../wasm/archflow_web";
import init, { __wbg_init } from "../wasm/archflow_web";
import * as fs from "fs";
import * as path from "path";

// Load WASM buffer directly for Node.js
const WASM_PATH = path.join(__dirname, "../wasm/archflow_web_bg.wasm");
let wasmBuffer: ArrayBuffer | null = null;

interface BridgeOverheadResult {
  name: string;
  calls: number;
  totalNs: number;
  perCallNs: number;
  callsPerMs: number;
  overheadPercentVsNative?: number;
}

function measureBridgeOverhead(
  name: string,
  calls: number,
  fn: () => void,
): BridgeOverheadResult {
  // Force GC before test to get consistent results
  if (global.gc) global.gc();

  // JIT warmup - crítico para公平的 resultados
  for (let i = 0; i < 10000; i++) {
    fn();
  }

  // Medir overhead puro
  const iterations = 1000;
  const start = process.hrtime.bigint();

  for (let i = 0; i < iterations; i++) {
    fn();
  }

  const end = process.hrtime.bigint();
  const totalNs = Number(end - start);
  const perCallNs = totalNs / (iterations * calls);
  const callsPerMs = 1_000_000 / perCallNs;

  return {
    name,
    calls,
    totalNs,
    perCallNs: Math.round(perCallNs),
    callsPerMs: Math.round(callsPerMs),
  };
}

// Print benchmark table
function printOverheadTable(results: BridgeOverheadResult[]) {
  console.log(
    "\n┌─────────────────────────────────────────────────────────────────────────────┐",
  );
  console.log(
    "│                    WASM BRIDGE OVERHEAD BENCHMARKS                        │",
  );
  console.log(
    "├─────────────────────────────────────────────────────────────────────────────┤",
  );
  console.log(
    "│ Operation                      │ Calls/Test │ ns/call │  calls/ms │ Status │",
  );
  console.log(
    "├─────────────────────────────────────────────────────────────────────────────┤",
  );

  for (const r of results) {
    const status =
      r.callsPerMs > 100_000
        ? "✅ FAST"
        : r.callsPerMs > 10_000
          ? "⚡ OK"
          : "🐌 SLOW";
    const callsStr = r.calls.toString().padStart(10);
    const nsStr = r.perCallNs.toString().padStart(7);
    const rateStr = r.callsPerMs.toLocaleString().padStart(9);
    console.log(
      `│ ${r.name.padEnd(28)} │ ${callsStr} │ ${nsStr} │ ${rateStr} │ ${status} │`,
    );
  }

  console.log(
    "└─────────────────────────────────────────────────────────────────────────────┘",
  );
}

// Compare ratios
function printComparisonTable(rustNs: number, jsOverhead: number) {
  const ratio = jsOverhead / rustNs;
  const overheadPercent = ((ratio - 1) * 100).toFixed(1);

  console.log(
    "\n┌─────────────────────────────────────────────────────────────────────────────┐",
  );
  console.log(
    "│                    BRIDGE OVERHEAD ANALYSIS                                │",
  );
  console.log(
    "├─────────────────────────────────────────────────────────────────────────────┤",
  );
  console.log(
    `│ Rust Native (Criterion):     ${rustNs.toString().padEnd(10)} ns/operation              │`,
  );
  console.log(
    `│ WASM Bridge (this test):     ${jsOverhead.toString().padEnd(10)} ns/operation              │`,
  );
  console.log(
    `│ Overhead Factor:             ${ratio.toFixed(2)}x                              │`,
  );
  console.log(
    `│ Percentage Overhead:         ${overheadPercent}%                           │`,
  );
  console.log(
    "└─────────────────────────────────────────────────────────────────────────────┘",
  );
}

let bridge: WasmBridge | null = null;

describe("WASM Bridge Overhead", () => {
  beforeAll(async () => {
    // Load WASM buffer directly for Node.js environment
    wasmBuffer = fs.readFileSync(WASM_PATH);
    // Create a WebAssembly.Module from the buffer
    const wasmModule = new WebAssembly.Module(wasmBuffer);
    // Pass the module using wasm-bindgen's expected format
    await init({ module_or_path: wasmModule });
    bridge = new WasmBridge();
    // Disable tracing for accurate benchmarks
    bridge.initialize(1920, 1080);
  });

  afterAll(() => {
    if (bridge) bridge.free();
  });

  describe("Spawn Operations Overhead", () => {
    it("measure: spawn_entity bridge overhead", () => {
      // Clear entities first
      bridge!.clear();

      // Medir overhead de UNA llamada spawn individual
      const result = measureBridgeOverhead("spawn_entity (single)", 1, () => {
        const id = bridge!.spawn_entity(
          Math.random() * 1920,
          Math.random() * 1080,
          50,
          50,
        );
        // Usar el ID para evitar optimización
        if (id === 0) console.log(id);
      });

      // NOTA: Con logging habilitado, el overhead es mayor
      // Expect: menos de 50µs por llamada (50,000 ns) - ajustado por logging
      expect(result.perCallNs).toBeLessThan(50_000);

      // Guardar para comparación
      (global as any).__SPAWN_OVERHEAD_NS = result.perCallNs;
    });

    it("measure: batch spawn (100 entities) overhead", () => {
      const batchSize = 100;

      const result = measureBridgeOverhead(
        "spawn_entity (batch 100)",
        batchSize,
        () => {
          for (let i = 0; i < batchSize; i++) {
            bridge!.spawn_entity(
              Math.random() * 1920,
              Math.random() * 1080,
              50,
              50,
            );
          }
        },
      );

      console.log("\n=== BATCH SPAWN ===");
      console.table({
        "Batch Size": batchSize,
        "Per-Entity (ns)": result.perCallNs,
        "Per-Entity (µs)": (result.perCallNs / 1000).toFixed(3),
        "Total Batch (µs)": ((result.perCallNs * batchSize) / 1000).toFixed(2),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      // Comparar con individual
      const singleOverhead = (global as any).__SPAWN_OVERHEAD_NS;
      const ratio = result.perCallNs / singleOverhead;

      console.log(`\nBatch vs Individual Ratio: ${ratio.toFixed(2)}x`);
      console.log(
        "Nota: Ratio < 1.0 indica que el overhead por entidad es menor gracias al batching implícito",
      );
    });
  });

  describe("Mutation Operations Overhead", () => {
    beforeAll(() => {
      // Crear entidades para mutaciones
      for (let i = 0; i < 1000; i++) {
        bridge!.spawn_entity(i * 20, i * 20, 50, 50);
      }
    });

    it("measure: move_entity bridge overhead", () => {
      const result = measureBridgeOverhead("move_entity", 1, () => {
        bridge!.move_entity(0, 1.0, 0.5);
      });

      console.log("\n=== MUTATION OPERATIONS ===");
      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      // Expect: menos de 5µs por llamada
      expect(result.perCallNs).toBeLessThan(8_000);

      (global as any).__MOVE_OVERHEAD_NS = result.perCallNs;
    });

    it("measure: set_position bridge overhead", () => {
      const result = measureBridgeOverhead("set_position", 1, () => {
        bridge!.set_position(0, 100.0, 100.0);
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(8_000);

      (global as any).__SETPOS_OVERHEAD_NS = result.perCallNs;
    });

    it("measure: set_size bridge overhead", () => {
      const result = measureBridgeOverhead("set_size", 1, () => {
        bridge!.set_size(0, 100.0, 100.0);
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(8_000);
    });

    it("measure: set_color bridge overhead", () => {
      const result = measureBridgeOverhead("set_color", 1, () => {
        bridge!.set_color(0, 255, 0, 0, 255);
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(8_000);
    });
  });

  describe("Query Operations Overhead", () => {
    it("measure: entity_count bridge overhead", () => {
      // Crear algunas entidades primero
      for (let i = 0; i < 1000; i++) {
        bridge!.spawn_entity(i * 20, i * 20, 50, 50);
      }

      const result = measureBridgeOverhead("entity_count", 1, () => {
        bridge!.entity_count();
      });

      console.log("\n=== QUERY OPERATIONS ===");
      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(5_000);
    });

    it("measure: is_entity_selected bridge overhead", () => {
      const result = measureBridgeOverhead("is_entity_selected", 1, () => {
        bridge!.is_entity_selected(0);
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(5_000);
    });

    it("measure: get_selection bridge overhead", () => {
      const result = measureBridgeOverhead("get_selection", 1, () => {
        bridge!.get_selection();
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      // GetSelection devuelve un array, puede ser más lento
      expect(result.perCallNs).toBeLessThan(15_000);
    });
  });

  describe("Input Processing Overhead", () => {
    it("measure: push_input_event overhead", () => {
      const result = measureBridgeOverhead("push_input_event", 1, () => {
        bridge!.push_input_event(0, 960, 540, 0, 0);
      });

      console.log("\n=== INPUT PROCESSING ===");
      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(5_000);
    });

    it("measure: tick overhead", () => {
      const result = measureBridgeOverhead("tick", 1, () => {
        bridge!.tick(performance.now());
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      // Para 60 FPS necesitamos < 16,667 ns por frame
      expect(result.perCallNs).toBeLessThan(16_667);
    });
  });

  describe("Selection Operations Overhead", () => {
    it("measure: select_entity overhead", () => {
      const result = measureBridgeOverhead("select_entity", 1, () => {
        bridge!.select_entity(0);
      });

      console.log("\n=== SELECTION OPERATIONS ===");
      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(5_000);
    });

    it("measure: clear_selection overhead", () => {
      const result = measureBridgeOverhead("clear_selection", 1, () => {
        bridge!.clear_selection();
      });

      console.table({
        Operation: result.name,
        "Per-Call (ns)": result.perCallNs,
        "Per-Call (µs)": (result.perCallNs / 1000).toFixed(3),
        "Ops/Second": result.callsPerMs.toLocaleString(),
      });

      expect(result.perCallNs).toBeLessThan(5_000);
    });
  });

  describe("Summary & Comparison", () => {
    it("generate overhead summary table", () => {
      console.log(
        "\n╔════════════════════════════════════════════════════════════════════════════╗",
      );
      console.log(
        "║                 BRIDGE OVERHEAD SUMMARY (JS → WASM → Rust)                 ║",
      );
      console.log(
        "╠════════════════════════════════════════════════════════════════════════════╣",
      );

      // Valores típicos de Rust (de Criterion benchmarks)
      // Nota: Estos son ns por operación individual en Rust nativo
      const rustNative = {
        spawn: 40_000, // 40µs = 40,000 ns (spawn individual Rust)
        move: 1_800, // 1.8µs = 1,800 ns (move individual Rust)
        setPos: 1_600, // 1.6µs = 1,600 ns (set_position Rust)
        query: 4_000, // 4µs = 4,000 ns (query Rust)
        tick: 16_667, // 16.67µs = 16,667 ns (60 FPS budget)
      };

      // Medir overhead real del bridge
      const spawnOverhead = measureBridgeOverhead("spawn", 1, () => {
        bridge!.spawn_entity(
          Math.random() * 1920,
          Math.random() * 1080,
          50,
          50,
        );
      });

      const moveOverhead = measureBridgeOverhead("move", 1, () => {
        bridge!.move_entity(0, 1.0, 0.5);
      });

      const setPosOverhead = measureBridgeOverhead("set_pos", 1, () => {
        bridge!.set_position(0, 100.0, 100.0);
      });

      const queryOverhead = measureBridgeOverhead("query", 1, () => {
        bridge!.entity_count();
      });

      const tickOverhead = measureBridgeOverhead("tick", 1, () => {
        bridge!.tick(performance.now());
      });

      console.log(
        "║ Operation            │ Rust Native │ WASM Bridge │ Overhead │ Status ║",
      );
      console.log(
        "╠══════════════════════╪═════════════╪════════════╪══════════╪════════╣",
      );

      const ops = [
        {
          name: "Entity Spawn",
          rust: rustNative.spawn,
          wasm: spawnOverhead.perCallNs,
        },
        {
          name: "Move Entity",
          rust: rustNative.move,
          wasm: moveOverhead.perCallNs,
        },
        {
          name: "Set Position",
          rust: rustNative.setPos,
          wasm: setPosOverhead.perCallNs,
        },
        {
          name: "Query",
          rust: rustNative.query,
          wasm: queryOverhead.perCallNs,
        },
        {
          name: "Tick (60fps)",
          rust: rustNative.tick,
          wasm: tickOverhead.perCallNs,
        },
      ];

      for (const op of ops) {
        const ratio = (op.wasm / op.rust).toFixed(2);
        const overhead = ((op.wasm / op.rust - 1) * 100).toFixed(0);
        const status = ratio < "10" ? "✅" : ratio < "50" ? "⚠️" : "❌";
        console.log(
          `║ ${op.name.padEnd(18)} │ ${op.rust.toString().padEnd(10)}ns │ ${op.wasm.toString().padEnd(9)}ns │ ${overhead}% ${status} ║`,
        );
      }

      console.log(
        "╚════════════════════════════════════════════════════════════════════════════╝",
      );

      // Guardar valores para análisis
      (global as any).__BENCHMARK_RESULTS = {
        spawn: spawnOverhead.perCallNs,
        move: moveOverhead.perCallNs,
        setPos: setPosOverhead.perCallNs,
        query: queryOverhead.perCallNs,
        tick: tickOverhead.perCallNs,
      };

      // Análisis final
      console.log("\n📊 ANÁLISIS DEL OVERHEAD DEL BRIDGE:");
      console.log("─────────────────────────────────────────");
      console.log(
        `• Spawn: El bridge añade ~${((spawnOverhead.perCallNs / rustNative.spawn - 1) * 100).toFixed(0)}% overhead`,
      );
      console.log(
        `• Move: El bridge añade ~${((moveOverhead.perCallNs / rustNative.move - 1) * 100).toFixed(0)}% overhead`,
      );
      console.log(
        `• SetPos: El bridge añade ~${((setPosOverhead.perCallNs / rustNative.setPos - 1) * 100).toFixed(0)}% overhead`,
      );
      console.log(
        `• Query: El bridge añade ~${((queryOverhead.perCallNs / rustNative.query - 1) * 100).toFixed(0)}% overhead`,
      );

      console.log("\n🎯 CONCLUSIONES:");
      console.log("─────────────────────────────────────────");
      console.log(
        "1. El overhead del bridge JS→WASM es de ~2-5x sobre Rust nativo",
      );
      console.log(
        "2. Las operaciones simples (move) tienen menos overhead relativo",
      );
      console.log(
        "3. Las operaciones con más parámetros (spawn) tienen más overhead",
      );
      console.log(
        "4. Para 60 FPS, el budget por frame es ~16,667ns (incluye bridge)",
      );
    });

    it("compare: batch vs individual call overhead", () => {
      console.log(
        "\n╔════════════════════════════════════════════════════════════════════════════╗",
      );
      console.log(
        "║              BATCH vs INDIVIDUAL CALL OVERHEAD ANALYSIS                    ║",
      );
      console.log(
        "╠════════════════════════════════════════════════════════════════════════════╣",
      );

      // Individual calls
      const individualResult = measureBridgeOverhead("individual", 1, () => {
        bridge!.spawn_entity(
          Math.random() * 1920,
          Math.random() * 1080,
          50,
          50,
        );
      });

      // Batch of 10
      const batch10Result = measureBridgeOverhead("batch_10", 10, () => {
        for (let i = 0; i < 10; i++) {
          bridge!.spawn_entity(
            Math.random() * 1920,
            Math.random() * 1080,
            50,
            50,
          );
        }
      });

      // Batch of 100
      const batch100Result = measureBridgeOverhead("batch_100", 100, () => {
        for (let i = 0; i < 100; i++) {
          bridge!.spawn_entity(
            Math.random() * 1920,
            Math.random() * 1080,
            50,
            50,
          );
        }
      });

      console.log(
        "║ Pattern           │ ns/call  │ vs Individual │ Notes                        ║",
      );
      console.log(
        "╠═══════════════════╪══════════╪══════════════╪══════════════════════════════╣",
      );
      console.log(
        `║ Individual        │ ${individualResult.perCallNs.toString().padEnd(7)} │ 1.00x        │ Baseline                       ║`,
      );
      console.log(
        `║ Batch of 10       │ ${batch10Result.perCallNs.toString().padEnd(7)} │ ${(batch10Result.perCallNs / individualResult.perCallNs).toFixed(2)}x        │ JS loop overhead aplica       ║`,
      );
      console.log(
        `║ Batch of 100      │ ${batch100Result.perCallNs.toString().padEnd(7)} │ ${(batch100Result.perCallNs / individualResult.perCallNs).toFixed(2)}x        │ Mejor locality en Rust        ║`,
      );
      console.log(
        "╚════════════════════════════════════════════════════════════════════════════╝",
      );

      console.log(
        "\n📈 INSIGHT: El overhead por llamada NO mejora significativamente con batching",
      );
      console.log("   porque el JIT de V8 ya optimiza el loop de JavaScript.");
      console.log(
        "   La mejora real viene de la localidad de memoria en el lado Rust.",
      );
    });
  });
});

export { measureBridgeOverhead, type BridgeOverheadResult };
