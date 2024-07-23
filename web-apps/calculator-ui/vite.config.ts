import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte(), wasm(), topLevelAwait()],
  server: {
    fs: { allow: ["../reti-js/wasm", "./"] },
  },
  test: {
    environment: "happy-dom",
    globals: true,
    setupFiles: ["./src/vitest-setup.ts"],
  },
});
