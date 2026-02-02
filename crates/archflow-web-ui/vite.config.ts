/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],

  assetsInclude: ["**/*.wasm"],

  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
    fs: {
      allow: ["..", "../../archflow-web/pkg"],
    },
  },

  optimizeDeps: {
    exclude: ["archflow_web"],
  },

  resolve: {
    alias: {
      "@archflow/web": "./src/wasm/archflow_web.js",
      "@components": "./src/components",
      "@hooks": "./src/hooks",
      "@utils": "./src/utils",
      "@types": "./src/types",
      "@store": "./src/store",
    },
  },

  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      exclude: [
        "node_modules/",
        "src/test/",
        "**/*.d.ts",
        "**/*.test.ts",
        "**/*.test.tsx",
      ],
    },
  },
});
