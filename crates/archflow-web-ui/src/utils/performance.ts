/**
 * Performance Monitoring Utilities
 *
 * Provides performance monitoring for FPS tracking, render times,
 * memory usage, and interaction latency. Useful for identifying
 * performance bottlenecks during development.
 *
 * Architecture Reference: EPIC-WEB-009
 */

/**
 * Performance metrics snapshot
 */
export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  memoryUsage: number;
  renderTime: number;
  interactionLatency: number;
}

/**
 * Performance monitor configuration
 */
interface PerformanceMonitorConfig {
  enableMemoryTracking: boolean;
  enableRenderTracking: boolean;
  enableFPSLogging: boolean;
  slowRenderThreshold: number; // milliseconds
}

const DEFAULT_CONFIG: PerformanceMonitorConfig = {
  enableMemoryTracking: true,
  enableRenderTracking: true,
  enableFPSLogging: false,
  slowRenderThreshold: 16, // Warn if render takes > 16ms (60fps threshold)
};

/**
 * Performance Monitor Class
 *
 * Tracks FPS, frame times, memory usage, and render performance.
 * Use in development to identify performance hotspots.
 */
class PerformanceMonitor {
  private config: PerformanceMonitorConfig;
  private metrics: PerformanceMetrics = {
    fps: 60,
    frameTime: 16.67,
    memoryUsage: 0,
    renderTime: 0,
    interactionLatency: 0,
  };

  private lastFpsUpdate = performance.now();
  private lastFrameTime = performance.now();
  private frames: number[] = [];
  private renderStartTime = 0;

  constructor(config: Partial<PerformanceMonitorConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Measure frame rate and frame time
   * Call this once per frame (typically in requestAnimationFrame)
   */
  measureFrame(): void {
    const now = performance.now();
    const delta = now - this.lastFrameTime;
    this.lastFrameTime = now;

    this.frames.push(delta);

    // Update FPS every second
    if (now - this.lastFpsUpdate > 1000) {
      const avgFrameTime =
        this.frames.reduce((a, b) => a + b, 0) / this.frames.length;
      this.metrics.fps = Math.round(1000 / avgFrameTime);
      this.metrics.frameTime = Number(avgFrameTime.toFixed(2));

      if (this.config.enableFPSLogging) {
        console.log(
          `[Performance] FPS: ${this.metrics.fps}, Frame Time: ${this.metrics.frameTime}ms`,
        );
      }

      this.frames = [];
      this.lastFpsUpdate = now;
    }
  }

  /**
   * Start measuring render time
   * Call at the start of a render cycle
   */
  startRender(): void {
    if (!this.config.enableRenderTracking) return;
    this.renderStartTime = performance.now();
  }

  /**
   * Stop measuring render time and log if slow
   * Call at the end of a render cycle
   */
  endRender(label?: string): void {
    if (!this.config.enableRenderTracking) return;

    const renderTime = performance.now() - this.renderStartTime;
    this.metrics.renderTime = Number(renderTime.toFixed(2));

    // Warn if render is slow
    if (renderTime > this.config.slowRenderThreshold) {
      const message = label
        ? `Slow render (${label}): ${renderTime.toFixed(2)}ms`
        : `Slow render: ${renderTime.toFixed(2)}ms`;
      console.warn(`[Performance] ${message}`);
    }
  }

  /**
   * Measure render time for a function
   * Wrapper that measures execution time of a function
   */
  measureRender<T>(label: string, fn: () => T): T {
    if (!this.config.enableRenderTracking) {
      return fn();
    }

    const start = performance.now();
    const result = fn();
    const end = performance.now();

    const renderTime = end - start;
    this.metrics.renderTime = Number(renderTime.toFixed(2));

    if (renderTime > this.config.slowRenderThreshold) {
      console.warn(
        `[Performance] Slow render (${label}): ${renderTime.toFixed(2)}ms`,
      );
    }

    return result;
  }

  /**
   * Measure interaction latency
   * Call this when handling user interactions (click, drag, etc.)
   */
  measureInteraction(label: string, fn: () => void): void {
    const start = performance.now();
    fn();
    const end = performance.now();

    const latency = end - start;
    this.metrics.interactionLatency = Number(latency.toFixed(2));

    if (latency > 50) {
      // Warn if interaction takes > 50ms (perceived as sluggish)
      console.warn(
        `[Performance] Slow interaction (${label}): ${latency.toFixed(2)}ms`,
      );
    }
  }

  /**
   * Get current memory usage (if available)
   * Only works in Chrome/Edge-based browsers
   */
  getMemoryUsage(): number {
    if (!this.config.enableMemoryTracking) return 0;

    // @ts-expect-error - performance.memory is non-standard but available in Chrome
    if (performance.memory) {
      // @ts-expect-error - performance.memory is non-standard
      return performance.memory.usedJSHeapSize;
    }
    return 0;
  }

  /**
   * Get memory usage in MB (human-readable)
   */
  getMemoryUsageMB(): number {
    const bytes = this.getMemoryUsage();
    return Number((bytes / 1024 / 1024).toFixed(2));
  }

  /**
   * Get current metrics snapshot
   */
  getMetrics(): PerformanceMetrics {
    return {
      ...this.metrics,
      memoryUsage: this.getMemoryUsage(),
    };
  }

  /**
   * Log current metrics to console
   */
  logMetrics(label?: string): void {
    const metrics = this.getMetrics();
    const prefix = label ? `[Performance] ${label}` : "[Performance]";

    console.log(prefix, {
      fps: metrics.fps,
      frameTime: `${metrics.frameTime}ms`,
      renderTime: `${metrics.renderTime}ms`,
      interactionLatency: `${metrics.interactionLatency}ms`,
      memory: `${this.getMemoryUsageMB()}MB`,
    });
  }

  /**
   * Reset all metrics
   */
  reset(): void {
    this.metrics = {
      fps: 60,
      frameTime: 16.67,
      memoryUsage: 0,
      renderTime: 0,
      interactionLatency: 0,
    };
    this.frames = [];
    this.lastFpsUpdate = performance.now();
  }
}

/**
 * Global performance monitor instance
 * Disabled by default in production
 */
export const perfMonitor = new PerformanceMonitor({
  enableMemoryTracking: import.meta.env.DEV,
  enableRenderTracking: import.meta.env.DEV,
  enableFPSLogging: false,
  slowRenderThreshold: 16,
});

/**
 * React hook for performance monitoring
 *
 * @example
 * function MyComponent() {
 *   usePerformanceMonitor("MyComponent");
 *   return <div>...</div>;
 * }
 */
export function usePerformanceMonitor(
  label: string,
  enabled: boolean = import.meta.env.DEV,
) {
  const startTime = React.useRef<number>(0);

  React.useEffect(() => {
    if (!enabled) return;

    startTime.current = performance.now();

    return () => {
      if (startTime.current) {
        const renderTime = performance.now() - startTime.current;
        if (renderTime > 16) {
          console.warn(
            `[Performance] Slow render (${label}): ${renderTime.toFixed(2)}ms`,
          );
        }
      }
    };
  }, [label, enabled]);
}

/**
 * React hook for measuring component render count
 * Useful for detecting unnecessary re-renders
 *
 * @example
 * function MyComponent({ prop }) {
 *   useRenderCount("MyComponent");
 *   return <div>...</div>;
 * }
 */
export function useRenderCount(
  label: string,
  enabled: boolean = import.meta.env.DEV,
) {
  const renderCount = React.useRef(0);

  React.useEffect(() => {
    if (!enabled) return;

    renderCount.current += 1;
    console.log(
      `[Performance] ${label} rendered: ${renderCount.current} times`,
    );
  });
}

// Import React for hooks
import React from "react";
