import { defineConfig } from "vite";
import path from "path";

export default defineConfig({
  base: "./",
  resolve: {
    alias: {
      "@styles": path.resolve(__dirname, "./styles"),
    },
  },
  build: {
    outDir: "dist",
    sourcemap: true,
  },
  server: {
    port: 5174,
    host: true,
    fs: {
      allow: [path.resolve(__dirname)],
      deny: ["../target"],
    },
    watch: {
      ignored: ["**/target/**", "**/node_modules/**"],
    },
  },
});
