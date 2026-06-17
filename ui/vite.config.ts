import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri drives this dev server; keep the port fixed and stop clearing logs.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
});
