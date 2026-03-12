import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

import { readFileSync } from "fs";

const tauriConf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  define: {
    __APP_VERSION__: JSON.stringify(tauriConf.version),
  },

  // Prevent vite from obscuring Rust errors
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
