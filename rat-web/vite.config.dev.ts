import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import path from "path";

export default defineConfig({
  plugins: [
    solidPlugin({
      hot: true,
      ssr: false
    })
  ],
  server: {
    port: 5173,
    strictPort: false,
    host: true
  },
  publicDir: 'public',
  resolve: {
    alias: {
      "~": path.resolve(__dirname, "./app"),
      "@solidjs/start": "solid-js",
      "solid-start": "solid-js"
    }
  },
  build: {
    target: "esnext"
  },
  optimizeDeps: {
    include: ["solid-js", "@solidjs/router", "@solidjs/meta", "@tanstack/solid-query"],
    exclude: ["solid-start"]
  }
});