/// <reference types="vitest/config" />
import react from "@vitejs/plugin-react";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@tremor": path.resolve(rootDir, "vendor/tremor"),
      "@ui": path.resolve(rootDir, "src/ui"),
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/vendor/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    outDir: "dist",
  },
});
