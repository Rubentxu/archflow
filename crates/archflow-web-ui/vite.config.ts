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
    // Use terser for better minification in production
    minify: "terser",
    terserOptions: {
      compress: {
        // Remove console logs in production
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ["console.log", "console.info", "console.debug"],
        // Optimize for size
        passes: 2,
        // Enable advanced optimizations
        unsafe: true,
      },
      mangle: {
        // Mangle property names starting with underscore
        properties: {
          regex: /^_/,
        },
        // Mangle top-level names
        toplevel: true,
      },
      format: {
        // Remove comments
        comments: false,
      },
    },
    // Generate source maps only for development
    sourcemap: false,
    // Report compressed size for better bundle size tracking
    reportCompressedSize: true,
    // Rollup options for code splitting
    rollupOptions: {
      output: {
        // Manual chunks for better tree-shaking and caching
        manualChunks: (id) => {
          // React and React DOM - most stable, rarely changes
          if (
            id.includes("node_modules/react/") ||
            id.includes("node_modules/react-dom/")
          ) {
            return "vendor-react";
          }
          // Animation library - changes infrequently
          if (id.includes("node_modules/framer-motion/")) {
            return "vendor-animation";
          }
          // State management and utilities
          if (
            id.includes("node_modules/zustand/") ||
            id.includes("node_modules/clsx/") ||
            id.includes("node_modules/tailwind-merge/")
          ) {
            return "vendor-utils";
          }
          // Form handling
          if (
            id.includes("node_modules/react-hook-form/") ||
            id.includes("node_modules/zod/") ||
            id.includes("@hookform/resolvers")
          ) {
            return "vendor-forms";
          }
          // Drag and drop
          if (id.includes("node_modules/@dnd-kit/")) {
            return "vendor-dnd";
          }
          // Icons - frequently updated
          if (id.includes("node_modules/lucide-react/")) {
            return "vendor-icons";
          }
          // Other vendor code
          if (id.includes("node_modules/")) {
            return "vendor-other";
          }
        },
        // Optimize chunk names for better caching
        chunkFileNames: "assets/chunk-[name]-[hash].js",
        entryFileNames: "assets/index-[hash].js",
        assetFileNames: "assets/[name]-[hash].[ext]",
      },
    },
    // Threshold for chunking (in KB) - warn if chunks are too large
    chunkSizeWarningLimit: 200,
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
