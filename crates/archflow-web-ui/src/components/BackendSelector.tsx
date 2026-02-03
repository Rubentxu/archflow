/**
 * Backend Selector Component
 *
 * UI for selecting and displaying graphics backend status.
 * Shows available backends, performance info, and allows selection.
 */

import React, { useState, useEffect } from "react";
import type { BackendInfo, GraphicsBackend } from "../types/wasm";

interface BackendSelectorProps {
  backendInfo: BackendInfo | null;
  selectedBackend: GraphicsBackend;
  onBackendChange: (backend: GraphicsBackend) => void;
  isInitializing?: boolean;
  error?: string | null;
}

/**
 * Badge component for backend availability
 */
function BackendBadge({
  available,
  label,
}: {
  available: boolean;
  label: string;
}) {
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "4px",
        padding: "2px 8px",
        borderRadius: "12px",
        fontSize: "11px",
        fontWeight: 500,
        backgroundColor: available ? "#10b981" : "#6b7280",
        color: "white",
      }}
    >
      {available ? "✓" : "✗"} {label}
    </span>
  );
}

/**
 * Performance info display
 */
function PerformanceInfo({ performance }: { performance: BackendInfo["performance"] }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "4px",
        fontSize: "12px",
        color: "#9ca3af",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <span style={{ color: "#60a5fa" }}>WebGL2:</span>
        <span>{performance.webgl2}</span>
      </div>
      {performance.webgpu !== performance.webgl2 && (
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span style={{ color: "#f59e0b" }}>WebGPU:</span>
          <span>{performance.webgpu}</span>
        </div>
      )}
    </div>
  );
}

export function BackendSelector({
  backendInfo,
  selectedBackend,
  onBackendChange,
  isInitializing = false,
  error,
}: BackendSelectorProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (!backendInfo) {
    return (
      <div
        style={{
          padding: "8px 12px",
          backgroundColor: "#1f2937",
          borderRadius: "8px",
          fontSize: "12px",
          color: "#9ca3af",
        }}
      >
        Detecting available backends...
      </div>
    );
  }

  return (
    <div
      style={{
        padding: "12px",
        backgroundColor: "#1f2937",
        borderRadius: "8px",
        fontSize: "13px",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: isExpanded ? "12px" : 0,
          cursor: "pointer",
        }}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <rect x="2" y="2" width="20" height="20" rx="2" />
            <circle cx="12" cy="12" r="3" />
          </svg>
          <span style={{ fontWeight: 500, color: "#f3f4f6" }}>Graphics Backend</span>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <BackendBadge available={backendInfo.webgl2} label="WebGL2" />
          <BackendBadge available={backendInfo.webgpu} label="WebGPU" />
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            style={{
              transform: isExpanded ? "rotate(180deg)" : "rotate(0deg)",
              transition: "transform 0.2s",
            }}
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </div>
      </div>

      {/* Expanded content */}
      {isExpanded && (
        <div
          style={{
            paddingTop: "12px",
            borderTop: "1px solid #374151",
          }}
        >
          {/* Backend selection */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "8px",
              marginBottom: "12px",
            }}
          >
            <label
              style={{
                fontSize: "12px",
                fontWeight: 500,
                color: "#d1d5db",
              }}
            >
              Select Backend:
            </label>
            <div
              style={{
                display: "flex",
                gap: "8px",
              }}
            >
              {/* WebGL2 option */}
              <button
                onClick={() => onBackendChange("webgl2")}
                disabled={!backendInfo.webgl2 || isInitializing}
                style={{
                  flex: 1,
                  padding: "8px 12px",
                  borderRadius: "6px",
                  border: "none",
                  cursor:
                    backendInfo.webgl2 && !isInitializing
                      ? "pointer"
                      : "not-allowed",
                  backgroundColor:
                    selectedBackend === "webgl2" ? "#3b82f6" : "#374151",
                  color: "white",
                  fontSize: "12px",
                  fontWeight: 500,
                  opacity: backendInfo.webgl2 ? 1 : 0.5,
                  transition: "all 0.2s",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                  WebGL2
                </div>
                <div
                  style={{
                    fontSize: "10px",
                    opacity: 0.7,
                    marginTop: "2px",
                  }}
                >
                  Universal support
                </div>
              </button>

              {/* WebGPU option */}
              <button
                onClick={() => onBackendChange("webgpu")}
                disabled={!backendInfo.webgpu || isInitializing}
                style={{
                  flex: 1,
                  padding: "8px 12px",
                  borderRadius: "6px",
                  border: "none",
                  cursor:
                    backendInfo.webgpu && !isInitializing
                      ? "pointer"
                      : "not-allowed",
                  backgroundColor:
                    selectedBackend === "webgpu" ? "#f59e0b" : "#374151",
                  color: "white",
                  fontSize: "12px",
                  fontWeight: 500,
                  opacity: backendInfo.webgpu ? 1 : 0.5,
                  transition: "all 0.2s",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
                  </svg>
                  WebGPU
                </div>
                <div
                  style={{
                    fontSize: "10px",
                    opacity: 0.7,
                    marginTop: "2px",
                  }}
                >
                  High performance
                </div>
              </button>
            </div>
          </div>

          {/* Performance info */}
          <div
            style={{
              padding: "8px",
              backgroundColor: "#111827",
              borderRadius: "6px",
              marginBottom: "8px",
            }}
          >
            <div
              style={{
                fontSize: "11px",
                fontWeight: 500,
                color: "#6b7280",
                marginBottom: "4px",
              }}
            >
              Performance
            </div>
            <PerformanceInfo performance={backendInfo.performance} />
          </div>

          {/* Status */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              fontSize: "12px",
            }}
          >
            {isInitializing ? (
              <>
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  style={{ animation: "spin 1s linear infinite" }}
                >
                  <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                </svg>
                <span style={{ color: "#f59e0b" }}>Initializing {selectedBackend}...</span>
              </>
            ) : error ? (
              <>
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span style={{ color: "#ef4444" }}>{error}</span>
              </>
            ) : (
              <>
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                  <polyline points="22 4 12 14.01 9 11.01" />
                </svg>
                <span style={{ color: "#10b981" }}>
                  Using {selectedBackend === "webgl2" ? "WebGL2" : "WebGPU"}
                </span>
              </>
            )}
          </div>
        </div>
      )}

      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default BackendSelector;
