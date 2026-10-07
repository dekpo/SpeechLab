import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Fixed port because the Tauri dev window points at it (see src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1430, strictPort: true },
});
