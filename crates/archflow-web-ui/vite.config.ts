import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // WASM support
  assetsInclude: ["**/*.wasm"],
  server: {
    headers: {
      // Enable SharedArrayBuffer for cross-origin isolation
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
    fs: {
      // Allow serving files from workspace root and archflow-web pkg
      allow: ["..", "../../archflow-web/pkg"],
    },
  },
  optimizeDeps: {
    // Exclude WASM from optimization
    exclude: ["archflow_web"],
  },
  resolve: {
    alias: {
      // Map archflow_web to the compiled WASM package
      "@archflow/web": "../../archflow-web/pkg",
    },
  },
});
