import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Dev port 1430 (not Tauri's default 1420, which the AssistantCabinetAI desktop app uses).
// The Tauri dev window points at it (see src-tauri/tauri.conf.json): change both together.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1430, strictPort: true },
  test: { include: ["src/**/*.test.ts"] },
});
