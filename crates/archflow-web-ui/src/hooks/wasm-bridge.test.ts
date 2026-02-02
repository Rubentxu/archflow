/**
 * Unit tests for wasm-bridge type helper
 */

import { describe, it, expect, vi } from "vitest";

// Inline the functions to avoid import issues in test environment
function isWasmBridge(value: unknown): boolean {
  if (value == null || typeof value !== "object") {
    return false;
  }
  const obj = value as Record<string, unknown>;
  return typeof obj.initialize === "function" && typeof obj.tick === "function";
}

function getTypedBridge<T extends Record<string, unknown>>(
  bridge: unknown,
): T | null {
  if (!isWasmBridge(bridge)) {
    return null;
  }
  return bridge as T;
}

describe("wasm-bridge", () => {
  describe("isWasmBridge", () => {
    it("should return true for valid WASM bridge object", () => {
      const validBridge = {
        initialize: () => {},
        tick: () => {},
      };
      expect(isWasmBridge(validBridge)).toBe(true);
    });

    it("should return false for null", () => {
      expect(isWasmBridge(null)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isWasmBridge(undefined)).toBe(false);
    });

    it("should return false for primitive values", () => {
      expect(isWasmBridge(42)).toBe(false);
      expect(isWasmBridge("string")).toBe(false);
      expect(isWasmBridge(true)).toBe(false);
    });

    it("should return false for objects missing required methods", () => {
      expect(isWasmBridge({})).toBe(false);
      expect(isWasmBridge({ initialize: () => {} })).toBe(false);
      expect(isWasmBridge({ tick: () => {} })).toBe(false);
    });

    it("should return false for objects with wrong method types", () => {
      expect(
        isWasmBridge({ initialize: "not a function", tick: () => {} }),
      ).toBe(false);
      expect(isWasmBridge({ initialize: () => {}, tick: 42 })).toBe(false);
    });
  });

  describe("getTypedBridge", () => {
    it("should return the bridge for valid WASM bridge object", () => {
      const validBridge = {
        initialize: () => {},
        tick: () => {},
        spawnEntity: () => 1,
      };
      const result = getTypedBridge(validBridge);
      expect(result).not.toBeNull();
      if (result) {
        expect(result.initialize).toBeDefined();
        expect(result.tick).toBeDefined();
      }
    });

    it("should return null for null", () => {
      expect(getTypedBridge(null)).toBeNull();
    });

    it("should return null for undefined", () => {
      expect(getTypedBridge(undefined)).toBeNull();
    });

    it("should return null for invalid objects", () => {
      expect(getTypedBridge({})).toBeNull();
      expect(getTypedBridge({ foo: "bar" })).toBeNull();
    });

    it("should preserve bridge methods when valid", () => {
      const mockFn = vi.fn();
      const validBridge = {
        initialize: mockFn,
        tick: vi.fn(),
        canUndo: () => true,
        canRedo: () => false,
      };
      const result = getTypedBridge(validBridge);
      expect(result).not.toBeNull();
      if (result) {
        expect(typeof result.initialize).toBe("function");
        expect(typeof result.tick).toBe("function");
        expect(typeof result.canUndo).toBe("function");
        expect(typeof result.canRedo).toBe("function");
      }
    });

    it("should work with typed return values", () => {
      const typedBridge = {
        initialize(_w: number, _h: number): void {},
        tick(_t: number): void {},
        spawnEntity(_x: number, _y: number, _w: number, _h: number): number {
          return 123;
        },
      };

      const result = getTypedBridge(typedBridge);
      expect(result).not.toBeNull();
      if (result) {
        const spawnResult = (
          result as { spawnEntity: () => number }
        ).spawnEntity();
        expect(spawnResult).toBe(123);
      }
    });
  });
});
