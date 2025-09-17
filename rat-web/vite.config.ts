import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  server: {
    port: 5173,
    strictPort: false,
    host: true
  },
  build: {
    target: "esnext"
  }
});

