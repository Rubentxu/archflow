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

  // Bundle optimization
  build: {
    // Enable CSS code splitting
    cssCodeSplit: true,
    // Minimize rollup output
    minify: "esbuild",
    // Generate source maps only for production builds
    sourcemap: false,
    // Rollup options for code splitting
    rollupOptions: {
      // Output configuration
      output: {
        // Manual chunks for better tree-shaking
        manualChunks: {
          // Separate vendor chunks
          "vendor-react": ["react", "react-dom"],
          "vendor-animation": ["framer-motion"],
          "vendor-utils": ["zustand", "clsx", "tailwind-merge"],
          "vendor-forms": ["react-hook-form", "zod", "@hookform/resolvers"],
          "vendor-dnd": [
            "@dnd-kit/core",
            "@dnd-kit/utilities",
            "@dnd-kit/modifiers",
          ],
          "vendor-icons": ["lucide-react"],
        },
        // Optimize chunk names
        chunkFileNames: "assets/chunk-[name]-[hash].js",
        entryFileNames: "assets/index-[hash].js",
        assetFileNames: "assets/[name]-[hash].[ext]",
      },
    },
    // Threshold for chunking (in bytes)
    chunkSizeWarningLimit: 500,
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
