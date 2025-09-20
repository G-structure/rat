import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import path from "path";

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 5173
  },
  resolve: {
    alias: {
      "~": path.resolve(__dirname, "./app")
    }
  },
  build: {
    target: "esnext"
  }
});