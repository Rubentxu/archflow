/**
 * Performance Monitoring Hooks and Utilities
 *
 * Real-time performance tracking for the WASM engine
 * with metrics aggregation and alerting.
 */

import { useCallback, useRef, useState, useEffect } from "react";
import type { WasmBridge } from "../wasm/archflow_web";

/**
 * Performance metrics collected per frame
 */
export interface FrameMetrics {
  frameTime: number;      // ms
  fps: number;
  entitiesRendered: number;
  drawCalls: number;
  memoryUsage: number;   // bytes
  inputLatency: number;   // ms
  physicsTime: number;    // ms
  renderTime: number;     // ms
}

/**
 * Aggregated performance statistics
 */
export interface PerformanceStats {
  fps: {
    current: number;
    avg: number;
    min: number;
    max: number;
  };
  frameTime: {
    avg: number;
    p50: number;
    p95: number;
    p99: number;
  };
  entities: {
    count: number;
    added: number;
    removed: number;
  };
  memory: {
    used: number;
    delta: number;
  };
}

/**
 * Performance monitoring configuration
 */
interface PerformanceMonitorConfig {
  /** Sample window size for statistics */
  windowSize: number;
  /** Enable memory tracking (requires performance.memory) */
  trackMemory: boolean;
  /** Alert threshold for low FPS */
  lowFpsThreshold: number;
  /** Alert threshold for high frame time */
  highFrameTimeThreshold: number;
}

/**
 * Performance monitor hook
 */
export function usePerformanceMonitor(
  bridge: WasmBridge | null,
  config: Partial<PerformanceMonitorConfig> = {},
) {
  const {
    windowSize = 300,      // 5 seconds at 60 FPS
    trackMemory = true,
    lowFpsThreshold = 30,
    highFrameTimeThreshold = 33, // >33ms = <30 FPS
  } = config;

  const [stats, setStats] = useState<PerformanceStats>({
    fps: { current: 0, avg: 0, min: Infinity, max: 0 },
    frameTime: { avg: 0, p50: 0, p95: 0, p99: 0 },
    entities: { count: 0, added: 0, removed: 0 },
    memory: { used: 0, delta: 0 },
  });

  const [alerts, setAlerts] = useState<string[]>([]);

  // Ring buffers for metrics
  const fpsBuffer = useRef<number[]>([]);
  const frameTimeBuffer = useRef<number[]>([]);
  const lastFrameTime = useRef(performance.now());
  const frameCount = useRef(0);
  const entityCount = useRef(0);
  const frameCallback = useRef<((metrics: FrameMetrics) => void) | null>(null);

  /**
   * Record a new frame
   */
  const recordFrame = useCallback((customMetrics?: Partial<FrameMetrics>) => {
    if (!bridge) return;

    const now = performance.now();
    const frameTime = now - lastFrameTime.current;
    lastFrameTime.current = now;

    // Calculate FPS
    frameCount.current++;
    const fps = 1000 / frameTime;

    // Get entity count from WASM
    const entityCountValue = bridge.get_entity_count?.() ?? entityCount.current;

    // Get memory if available
    const memoryUsed = trackMemory
      ? (performance.memory?.usedJSHeapSize ?? 0)
      : 0;

    const metrics: FrameMetrics = {
      frameTime,
      fps,
      entitiesRendered: entityCountValue,
      drawCalls: customMetrics?.drawCalls ?? 0,
      memoryUsage: memoryUsed,
      inputLatency: customMetrics?.inputLatency ?? 0,
      physicsTime: customMetrics?.physicsTime ?? 0,
      renderTime: customMetrics?.renderTime ?? 0,
    };

    // Add to buffers
    fpsBuffer.current.push(fps);
    frameTimeBuffer.current.push(frameTime);
    entityCount.current = entityCountValue;

    // Trim buffers
    if (fpsBuffer.current.length > windowSize) {
      fpsBuffer.current.shift();
    }
    if (frameTimeBuffer.current.length > windowSize) {
      frameTimeBuffer.current.shift();
    }

    // Calculate statistics
    const fpsArr = fpsBuffer.current;
    const ftArr = frameTimeBuffer.current;

    const avgFps = fpsArr.reduce((a, b) => a + b, 0) / fpsArr.length;
    const minFps = Math.min(...fpsArr);
    const maxFps = Math.max(...fpsArr);

    ftArr.sort((a, b) => a - b);
    const p50 = ftArr[Math.floor(ftArr.length * 0.50)] ?? 0;
    const p95 = ftArr[Math.floor(ftArr.length * 0.95)] ?? 0;
    const p99 = ftArr[Math.floor(ftArr.length * 0.99)] ?? 0;
    const avgFrameTime = ftArr.reduce((a, b) => a + b, 0) / ftArr.length;

    // Check for alerts
    const newAlerts: string[] = [];
    if (avgFps < lowFpsThreshold) {
      newAlerts.push(`Low FPS: ${avgFps.toFixed(1)} (< ${lowFpsThreshold})`);
    }
    if (avgFrameTime > highFrameTimeThreshold) {
      newAlerts.push(`High Frame Time: ${avgFrameTime.toFixed(1)}ms (> ${highFrameTimeThreshold}ms)`);
    }

    // Update state
    setStats({
      fps: { current: fps, avg: avgFps, min: minFps, max: maxFps },
      frameTime: { avg: avgFrameTime, p50, p95, p99 },
      entities: {
        count: entityCountValue,
        added: 0,
        removed: 0,
      },
      memory: {
        used: memoryUsed,
        delta: 0,
      },
    });

    if (newAlerts.length > 0) {
      setAlerts(newAlerts);
    }

    // Callback
    if (frameCallback.current) {
      frameCallback.current(metrics);
    }
  }, [bridge, windowSize, lowFpsThreshold, highFrameTimeThreshold, trackMemory]);

  /**
   * Start the monitoring loop
   */
  const startMonitoring = useCallback(() => {
    let animationId: number;

    const loop = () => {
      recordFrame();
      animationId = requestAnimationFrame(loop);
    };

    animationId = requestAnimationFrame(loop);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [recordFrame]);

  /**
   * Get entity count
   */
  const getEntityCount = useCallback(() => {
    if (!bridge) return 0;
    return bridge.get_entity_count?.() ?? 0;
  }, [bridge]);

  /**
   * Subscribe to frame metrics
   */
  const onFrame = useCallback((callback: (metrics: FrameMetrics) => void) => {
    frameCallback.current = callback;
    return () => { frameCallback.current = null; };
  }, []);

  /**
   * Clear statistics
   */
  const clearStats = useCallback(() => {
    fpsBuffer.current = [];
    frameTimeBuffer.current = [];
    setStats({
      fps: { current: 0, avg: 0, min: Infinity, max: 0 },
      frameTime: { avg: 0, p50: 0, p95: 0, p99: 0 },
      entities: { count: 0, added: 0, removed: 0 },
      memory: { used: 0, delta: 0 },
    });
  }, []);

  return {
    stats,
    alerts,
    recordFrame,
    startMonitoring,
    getEntityCount,
    onFrame,
    clearStats,
  };
}

/**
 * FPS Counter hook for simple FPS display
 */
export function useFpsCounter(bridge: WasmBridge | null, enabled = true) {
  const [fps, setFps] = useState(0);
  const frameTimesRef = useRef<number[]>([]);
  const lastTimeRef = useRef(performance.now());
  const frameCountRef = useRef(0);

  useEffect(() => {
    if (!enabled || !bridge) return;

    let animationId: number;

    const update = () => {
      const now = performance.now();
      frameCountRef.current++;

      // Update FPS every second
      if (now - lastTimeRef.current >= 1000) {
        setFps(frameCountRef.current);
        frameCountRef.current = 0;
        lastTimeRef.current = now;
      }

      animationId = requestAnimationFrame(update);
    };

    animationId = requestAnimationFrame(update);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [bridge, enabled]);

  return fps;
}

/**
 * Frame time history for graphing
 */
export function useFrameTimeHistory(maxPoints = 120) {
  const [history, setHistory] = useState<{ time: number; value: number }[]>([]);

  const record = useCallback((frameTime: number) => {
    setHistory(prev => {
      const now = Date.now();
      const newEntry = { time: now, value: frameTime };
      const updated = [...prev, newEntry];

      // Keep only recent entries
      const cutoff = now - 2000; // 2 seconds
      return updated.filter(e => e.time > cutoff);
    });
  }, []);

  return { history, record };
}

/**
 * Performance reporter for automated testing
 */
export class PerformanceReporter {
  private samples: Map<string, number[]> = new Map();
  private startTime: number = 0;

  start() {
    this.startTime = performance.now();
    this.samples.clear();
  }

  sample(name: string, value: number) {
    if (!this.samples.has(name)) {
      this.samples.set(name, []);
    }
    this.samples.get(name)!.push(value);
  }

  report(): Record<string, { avg: number; min: number; max: number; p95: number; count: number }> {
    const report: Record<string, { avg: number; min: number; max: number; p95: number; count: number }> = {};

    for (const [name, values] of this.samples) {
      values.sort((a, b) => a - b);
      const sum = values.reduce((a, b) => a + b, 0);
      report[name] = {
        avg: sum / values.length,
        min: values[0],
        max: values[values.length - 1],
        p95: values[Math.floor(values.length * 0.95)] ?? 0,
        count: values.length,
      };
    }

    return report;
  }

  summary() {
    const report = this.report();
    console.group("📊 Performance Report");
    console.table(report);
    console.groupEnd();
    return report;
  }
}

/**
 * Benchmark decorator for timing function executions
 */
export function benchmark(name: string, reporter: PerformanceReporter) {
  return function <T extends (...args: any[]) => any>(
    fn: T
  ): T {
    return ((...args: Parameters<T>) => {
      const start = performance.now();
      const result = fn(...args);
      const duration = performance.now() - start;
      reporter.sample(name, duration);
      return result;
    }) as T;
  };
}

export type { PerformanceMonitorConfig };
