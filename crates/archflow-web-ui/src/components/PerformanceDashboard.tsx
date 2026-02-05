/**
 * Performance Dashboard Component
 *
 * Real-time performance visualization for the WASM engine.
 */

import React, { useEffect, useRef } from "react";
import { usePerformanceMonitor, useFpsCounter } from "../hooks/usePerformanceMonitor";

interface PerformanceDashboardProps {
  bridge: any | null;
  visible?: boolean;
  compact?: boolean;
}

export function PerformanceDashboard({ bridge, visible = true, compact = false }: PerformanceDashboardProps) {
  const { stats, alerts, startMonitoring, clearStats, getEntityCount } = usePerformanceMonitor(bridge);
  const fps = useFpsCounter(bridge, visible);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // FPS graph drawing
  useEffect(() => {
    if (!canvasRef.current || compact) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    // Clear
    ctx.fillStyle = "#1a1a2e";
    ctx.fillRect(0, 0, width, height);

    // Draw grid
    ctx.strokeStyle = "#2d2d44";
    ctx.lineWidth = 1;
    for (let i = 0; i < 5; i++) {
      const y = (height / 5) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw FPS line (mock data for visualization)
    const now = Date.now();
    ctx.strokeStyle = "#00ff88";
    ctx.lineWidth = 2;
    ctx.beginPath();

    for (let x = width - 1; x >= 0; x--) {
      const fpsValue = 55 + Math.sin((now / 1000 + x / 50) * 0.5) * 5;
      const y = height - (fpsValue / 60) * height;
      if (x === width - 1) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();

  }, [compact]);

  if (!visible) return null;

  return (
    <div style={{
      position: "fixed",
      top: compact ? "10px" : "10px",
      right: "10px",
      background: "rgba(26, 26, 46, 0.95)",
      border: "1px solid #2d2d44",
      borderRadius: "8px",
      padding: compact ? "8px 12px" : "16px",
      fontFamily: "monospace",
      fontSize: compact ? "11px" : "12px",
      color: "#e0e0e0",
      minWidth: compact ? "140px" : "280px",
      zIndex: 9999,
      backdropFilter: "blur(10px)",
    }}>
      {/* Header */}
      {!compact && (
        <div style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "12px",
          paddingBottom: "8px",
          borderBottom: "1px solid #2d2d44",
        }}>
          <span style={{ fontWeight: "bold", color: "#00ff88" }}>⚡ Performance</span>
          <button
            onClick={clearStats}
            style={{
              background: "none",
              border: "none",
              color: "#888",
              cursor: "pointer",
              fontSize: "10px",
            }}
          >
            [Reset]
          </button>
        </div>
      )}

      {/* FPS Display */}
      <div style={{
        display: "flex",
        alignItems: "baseline",
        gap: "8px",
        marginBottom: compact ? "4px" : "12px",
      }}>
        <span style={{
          fontSize: compact ? "18px" : "28px",
          fontWeight: "bold",
          color: fps >= 55 ? "#00ff88" : fps >= 30 ? "#ffaa00" : "#ff4444",
        }}>
          {fps}
        </span>
        <span style={{ color: "#888" }}>FPS</span>
      </div>

      {/* Frame Time */}
      {!compact && (
        <>
          {/* FPS Graph */}
          <div style={{ marginBottom: "12px" }}>
            <canvas
              ref={canvasRef}
              width={250}
              height={60}
              style={{ width: "100%", height: "60px", borderRadius: "4px" }}
            />
          </div>

          {/* Metrics Grid */}
          <div style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: "8px",
            marginBottom: "12px",
          }}>
            <Metric label="Frame" value={`${stats.frameTime.avg.toFixed(1)}ms`} />
            <Metric label="P95" value={`${stats.frameTime.p95.toFixed(1)}ms`} />
            <Metric label="Entities" value={stats.entities.count.toLocaleString()} />
            <Metric label="Memory" value={formatBytes(stats.memory.used)} />
          </div>

          {/* Detailed Stats */}
          <div style={{
            background: "rgba(0, 0, 0, 0.3)",
            borderRadius: "4px",
            padding: "8px",
            fontSize: "10px",
          }}>
            <div style={{ color: "#666", marginBottom: "4px" }}>FPS Range</div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span>Min: <span style={{ color: "#ff6b6b" }}>{stats.fps.min.toFixed(0)}</span></span>
              <span>Avg: <span style={{ color: "#4ecdc4" }}>{stats.fps.avg.toFixed(1)}</span></span>
              <span>Max: <span style={{ color: "#00ff88" }}>{stats.fps.max.toFixed(0)}</span></span>
            </div>
          </div>
        </>
      )}

      {/* Alerts */}
      {alerts.length > 0 && (
        <div style={{
          marginTop: "8px",
          padding: "8px",
          background: "rgba(255, 68, 68, 0.1)",
          border: "1px solid #ff4444",
          borderRadius: "4px",
          color: "#ff6b6b",
          fontSize: "10px",
        }}>
          {alerts.map((alert, i) => (
            <div key={i}>⚠️ {alert}</div>
          ))}
        </div>
      )}

      {/* Quick Actions */}
      {compact && (
        <div style={{ display: "flex", gap: "8px", marginTop: "4px" }}>
          <span style={{ color: "#666" }}>|</span>
          <span>{stats.entities.count.toLocaleString()} entities</span>
        </div>
      )}
    </div>
  );
}

/**
 * Simple metric display
 */
function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div style={{
      background: "rgba(0, 0, 0, 0.2)",
      padding: "6px 8px",
      borderRadius: "4px",
    }}>
      <div style={{ color: "#666", fontSize: "9px" }}>{label}</div>
      <div style={{ color: "#e0e0e0", fontWeight: "bold" }}>{value}</div>
    </div>
  );
}

/**
 * Format bytes to human readable
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

/**
 * Performance Summary Panel
 * For detailed reporting
 */
interface PerformanceSummaryProps {
  bridge: any | null;
}

export function PerformanceSummary({ bridge }: PerformanceSummaryProps) {
  const { stats, getEntityCount } = usePerformanceMonitor(bridge);

  return (
    <div style={{
      position: "fixed",
      bottom: "10px",
      right: "10px",
      background: "rgba(26, 26, 46, 0.95)",
      border: "1px solid #2d2d44",
      borderRadius: "8px",
      padding: "12px 16px",
      fontFamily: "monospace",
      fontSize: "12px",
      color: "#e0e0e0",
      zIndex: 9999,
    }}>
      <div style={{ fontWeight: "bold", marginBottom: "8px" }}>📊 Performance Summary</div>
      <div style={{ display: "grid", gap: "4px" }}>
        <div>FPS: <span style={{ color: "#00ff88" }}>{stats.fps.avg.toFixed(1)}</span></div>
        <div>Frame Time: <span style={{ color: "#4ecdc4" }}>{stats.frameTime.avg.toFixed(2)}ms</span></div>
        <div>P95: <span style={{ color: "#ffaa00" }}>{stats.frameTime.p95.toFixed(2)}ms</span></div>
        <div>Entities: <span style={{ color: "#e0e0e0" }}>{stats.entities.count.toLocaleString()}</span></div>
        <div>Memory: <span style={{ color: "#888" }}>{formatBytes(stats.memory.used)}</span></div>
      </div>
    </div>
  );
}

export default PerformanceDashboard;
