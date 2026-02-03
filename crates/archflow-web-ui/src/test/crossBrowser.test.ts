/**
 * Cross-Browser Compatibility Tests
 *
 * Tests for browser feature detection and compatibility.
 * Ensures the application works across Chrome, Firefox, Safari, and Edge.
 *
 * Architecture Reference: EPIC-WEB-009
 */

import { describe, it, expect } from "vitest";

describe("Browser Compatibility", () => {
  describe("Modern JavaScript Features", () => {
    it("should support optional chaining operator", () => {
      const obj = { a: { b: { c: 1 } } };
      expect(obj?.a?.b?.c).toBe(1);
      // Test optional chaining with undefined intermediate values
      const partialObj = { a: { b: undefined } };
      expect((partialObj as any)?.x?.y?.z).toBeUndefined();
    });

    it("should support nullish coalescing operator", () => {
      const nullVal = null;
      const undefinedVal = undefined;
      const zero = 0;
      const emptyString = "";

      expect(nullVal ?? "default").toBe("default");
      expect(undefinedVal ?? "default").toBe("default");
      expect(zero ?? "default").toBe(0); // 0 is not falsy for ??
      expect(emptyString ?? "default").toBe(""); // "" is not falsy for ??
    });

    it("should support BigInt", () => {
      expect(typeof BigInt("123")).toBe("bigint");
      const bigIntValue = BigInt(9007199254740991);
      expect(bigIntValue.toString()).toBe("9007199254740991");
    });

    it("should support Promise.allSettled", () => {
      const promises = [
        Promise.resolve(1),
        Promise.reject("error"),
        Promise.resolve(3),
      ];

      return Promise.allSettled(promises).then((results) => {
        expect(results).toHaveLength(3);
        expect(results[0].status).toBe("fulfilled");
        expect(results[1].status).toBe("rejected");
        expect(results[2].status).toBe("fulfilled");
      });
    });

    it("should support Object.fromEntries", () => {
      const entries = [
        ["a", 1],
        ["b", 2],
        ["c", 3],
      ];
      const obj = Object.fromEntries(entries);
      expect(obj).toEqual({ a: 1, b: 2, c: 3 });
    });

    it("should support Array.prototype.flat", () => {
      const arr = [1, [2, [3, [4]]]];
      expect(arr.flat(2)).toEqual([1, 2, 3, [4]]);
    });

    it("should support Array.prototype.flatMap", () => {
      const arr = [1, 2, 3];
      const result = arr.flatMap((x) => [x, x * 2]);
      expect(result).toEqual([1, 2, 2, 4, 3, 6]);
    });
  });

  describe("Web APIs", () => {
    it("should support ResizeObserver", () => {
      expect(typeof ResizeObserver).toBe("function");
    });

    it("should support IntersectionObserver", () => {
      expect(typeof IntersectionObserver).toBe("function");
    });

    it("should support MutationObserver", () => {
      expect(typeof MutationObserver).toBe("function");
    });

    it("should support requestAnimationFrame", () => {
      expect(typeof requestAnimationFrame).toBe("function");
      expect(typeof cancelAnimationFrame).toBe("function");
    });

    it("should support Performance API", () => {
      expect(typeof performance.now).toBe("function");
      expect(typeof performance.getEntriesByType).toBe("function");
    });

    it("should support URL and URLSearchParams", () => {
      const url = new URL("https://example.com?foo=bar&baz=qux");
      expect(url.searchParams.get("foo")).toBe("bar");
      expect(url.searchParams.get("baz")).toBe("qux");
    });

    it("should support Clipboard API", () => {
      // Note: Clipboard API might not be available in all contexts (e.g., non-secure origins)
      // We just check if it exists
      expect(typeof navigator.clipboard).toBe("object");
    });
  });

  describe("Canvas 2D Features", () => {
    it("should support Canvas 2D context", () => {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      expect(ctx).toBeInstanceOf(CanvasRenderingContext2D);
    });

    it("should support Canvas 2D modern features", () => {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");

      if (ctx) {
        // Check for modern canvas features
        expect(typeof ctx.resetTransform).toBe("function");
        expect(typeof ctx.roundRect).toBe("function");
      }
    });

    it("should support devicePixelRatio", () => {
      expect(typeof window.devicePixelRatio).toBe("number");
      expect(window.devicePixelRatio).toBeGreaterThan(0);
    });
  });

  describe("Pointer Events", () => {
    it("should support pointer events", () => {
      const canvas = document.createElement("canvas");
      expect(typeof canvas.onpointerdown).toBe("object");
      expect(typeof canvas.onpointermove).toBe("object");
      expect(typeof canvas.onpointerup).toBe("object");
      expect(typeof canvas.onpointercancel).toBe("object");
    });
  });

  describe("EventTarget Features", () => {
    it("should support addEventListener with options", () => {
      const element = document.createElement("div");
      let called = false;

      element.addEventListener(
        "click",
        () => {
          called = true;
        },
        { passive: true, once: true },
      );

      expect(() => element.click()).not.toThrow();
      expect(called).toBe(true);
    });

    it("should support event delegation with composedPath", () => {
      const parent = document.createElement("div");
      const child = document.createElement("div");
      parent.appendChild(child);

      let eventPath: EventTarget[] = [];
      parent.addEventListener("click", (e) => {
        eventPath = e.composedPath();
      });

      child.click();
      expect(eventPath.length).toBeGreaterThan(0);
      expect(eventPath).toContain(child);
      expect(eventPath).toContain(parent);
    });
  });

  describe("DOM Features", () => {
    it("should support append and prepend methods", () => {
      const parent = document.createElement("div");
      const child1 = document.createElement("div");
      const child2 = document.createElement("div");

      parent.append(child1);
      parent.prepend(child2);

      expect(parent.children[0]).toBe(child2);
      expect(parent.children[1]).toBe(child1);
    });

    it("should support replaceChildren method", () => {
      const parent = document.createElement("div");
      parent.appendChild(document.createElement("div"));

      const newChild = document.createElement("div");
      parent.replaceChildren(newChild);

      expect(parent.children.length).toBe(1);
      expect(parent.children[0]).toBe(newChild);
    });

    it("should support before and after methods", () => {
      const parent = document.createElement("div");
      const child = document.createElement("div");
      parent.appendChild(child);

      const before = document.createElement("div");
      const after = document.createElement("div");

      child.before(before);
      child.after(after);

      expect(parent.children[0]).toBe(before);
      expect(parent.children[1]).toBe(child);
      expect(parent.children[2]).toBe(after);
    });
  });

  describe("CSS Features", () => {
    it("should support CSS.supports for modern features", () => {
      expect(CSS.supports("display", "grid")).toBe(true);
      expect(CSS.supports("display", "flex")).toBe(true);
      expect(CSS.supports("position", "sticky")).toBe(true);
    });

    it("should support CSS custom properties", () => {
      const element = document.createElement("div");
      element.style.setProperty("--test-color", "red");
      expect(element.style.getPropertyValue("--test-color")).toBe("red");
    });
  });

  describe("Storage APIs", () => {
    it("should support localStorage", () => {
      expect(typeof localStorage).toBe("object");
      expect(typeof localStorage.setItem).toBe("function");
      expect(typeof localStorage.getItem).toBe("function");
      expect(typeof localStorage.removeItem).toBe("function");
    });

    it("should support sessionStorage", () => {
      expect(typeof sessionStorage).toBe("object");
      expect(typeof sessionStorage.setItem).toBe("function");
      expect(typeof sessionStorage.getItem).toBe("function");
    });
  });

  describe("Type Conversions", () => {
    it("should support Number.isFinite", () => {
      expect(Number.isFinite(1)).toBe(true);
      expect(Number.isFinite(Infinity)).toBe(false);
      expect(Number.isFinite(NaN)).toBe(false);
    });

    it("should support Number.isNaN", () => {
      expect(Number.isNaN(NaN)).toBe(true);
      expect(Number.isNaN(1)).toBe(false);
      expect(Number.isNaN("NaN")).toBe(false);
    });

    it("should support Number.isInteger", () => {
      expect(Number.isInteger(1)).toBe(true);
      expect(Number.isInteger(1.0)).toBe(true);
      expect(Number.isInteger(1.1)).toBe(false);
      expect(Number.isInteger(NaN)).toBe(false);
    });
  });

  describe("String Features", () => {
    it("should support String.prototype.includes", () => {
      expect("hello world".includes("world")).toBe(true);
      expect("hello world".includes("goodbye")).toBe(false);
    });

    it("should support String.prototype.startsWith", () => {
      expect("hello world".startsWith("hello")).toBe(true);
      expect("hello world".startsWith("world")).toBe(false);
    });

    it("should support String.prototype.endsWith", () => {
      expect("hello world".endsWith("world")).toBe(true);
      expect("hello world".endsWith("hello")).toBe(false);
    });

    it("should support String.prototype.repeat", () => {
      expect("abc".repeat(3)).toBe("abcabcabc");
    });

    it("should support String.prototype.trimStart and trimEnd", () => {
      expect("  hello  ".trimStart()).toBe("hello  ");
      expect("  hello  ".trimEnd()).toBe("  hello");
      expect("  hello  ".trim()).toBe("hello");
    });
  });

  describe("Performance Optimization Features", () => {
    it("should support queueMicrotask", () => {
      expect(typeof queueMicrotask).toBe("function");
    });

    it("should support structuredClone for deep copying", () => {
      const obj = { a: 1, b: { c: 2 } };
      const cloned = structuredClone(obj);
      expect(cloned).toEqual(obj);
      expect(cloned).not.toBe(obj);
      expect(cloned.b).not.toBe(obj.b);
    });
  });

  describe("Async Iteration", () => {
    it("should support async/await syntax", async () => {
      const asyncFn = async () => {
        return Promise.resolve(42);
      };

      const result = await asyncFn();
      expect(result).toBe(42);
    });

    it("should support for await...of syntax", async () => {
      async function* asyncGenerator() {
        yield 1;
        yield 2;
        yield 3;
      }

      const results: number[] = [];
      for await (const value of asyncGenerator()) {
        results.push(value);
      }

      expect(results).toEqual([1, 2, 3]);
    });
  });

  describe("React-Compatible Features", () => {
    it("should support classList API used by React", () => {
      const element = document.createElement("div");
      element.classList.add("class1", "class2");
      expect(element.classList.contains("class1")).toBe(true);

      element.classList.remove("class1");
      expect(element.classList.contains("class1")).toBe(false);

      element.classList.toggle("class3");
      expect(element.classList.contains("class3")).toBe(true);

      element.classList.toggle("class3");
      expect(element.classList.contains("class3")).toBe(false);
    });

    it("should support dataset attribute", () => {
      const element = document.createElement("div");
      element.dataset.testValue = "hello";
      expect(element.dataset.testValue).toBe("hello");
    });
  });

  describe("WASM Compatibility", () => {
    it("should support WebAssembly", () => {
      expect(typeof WebAssembly).toBe("object");
      expect(typeof WebAssembly.instantiate).toBe("function");
      expect(typeof WebAssembly.compile).toBe("function");
    });

    it("should support Uint8Array for WASM memory", () => {
      const buffer = new ArrayBuffer(8);
      const uint8 = new Uint8Array(buffer);
      uint8[0] = 255;
      expect(uint8[0]).toBe(255);
    });
  });

  describe("Touch and Pointer Events for Mobile", () => {
    it("should support touch events", () => {
      // Touch events might not be available on desktop
      const hasTouch = "ontouchstart" in window;
      if (hasTouch) {
        expect(typeof TouchEvent).toBe("function");
      }
    });

    it("should support pointer events for unified input", () => {
      expect(typeof PointerEvent).toBe("function");
      const event = new PointerEvent("pointerdown", {
        clientX: 100,
        clientY: 100,
      });
      expect(event.clientX).toBe(100);
      expect(event.clientY).toBe(100);
    });
  });
});

describe("Performance Benchmarks", () => {
  it("should render simple component quickly", () => {
    const start = performance.now();

    // Simulate simple rendering
    const div = document.createElement("div");
    for (let i = 0; i < 100; i++) {
      const child = document.createElement("span");
      child.textContent = `Item ${i}`;
      div.appendChild(child);
    }

    const end = performance.now();
    const renderTime = end - start;

    // Should render 100 elements in less than 50ms
    expect(renderTime).toBeLessThan(50);
  });

  it("should handle rapid state changes efficiently", () => {
    const updates: number[] = [];
    const start = performance.now();

    // Simulate rapid updates
    for (let i = 0; i < 1000; i++) {
      const updateStart = performance.now();
      // Simulate simple update
      const value = i * 2;
      updates.push(value);
      const updateEnd = performance.now();
      const updateTime = updateEnd - updateStart;

      // Each update should be very fast
      expect(updateTime).toBeLessThan(1);
    }

    const end = performance.now();
    const totalTime = end - start;

    // All 1000 updates should complete in less than 100ms
    expect(totalTime).toBeLessThan(100);
    expect(updates).toHaveLength(1000);
  });

  it("should measure FPS during animation loop", async () => {
    return new Promise<void>((resolve) => {
      const frames: number[] = [];
      let frameCount = 0;
      const maxFrames = 10;

      let lastTime = performance.now();

      function measureFrame() {
        const currentTime = performance.now();
        const frameTime = currentTime - lastTime;
        lastTime = currentTime;

        frames.push(frameTime);
        frameCount++;

        if (frameCount < maxFrames) {
          requestAnimationFrame(measureFrame);
        } else {
          // Calculate average frame time
          const avgFrameTime =
            frames.reduce((a, b) => a + b, 0) / frames.length;
          const fps = 1000 / avgFrameTime;

          // Should maintain at least 30 FPS
          expect(fps).toBeGreaterThanOrEqual(30);

          // Frame time should be reasonable
          expect(avgFrameTime).toBeLessThan(50);

          resolve();
        }
      }

      requestAnimationFrame(measureFrame);
    });
  });
});
