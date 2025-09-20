import { defineConfig } from "vite";
import solid from "solid-start/vite";
import cloudflare from "solid-start-cloudflare-pages";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  plugins: [
    solid({ adapter: cloudflare() }),
    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["icons/*", "offline.html"],
      manifest: {
        name: "RAT Mobile IDE",
        short_name: "RAT IDE",
        description: "AI-powered mobile code editor",
        start_url: "/",
        display: "standalone",
        orientation: "portrait",
        background_color: "#0b0b0b",
        theme_color: "#0b0b0b",
        icons: [
          {
            src: "/icons/pwa-192.png",
            sizes: "192x192",
            type: "image/png"
          },
          {
            src: "/icons/pwa-512.png", 
            sizes: "512x512",
            type: "image/png"
          },
          {
            src: "/icons/maskable-1024.png",
            sizes: "1024x1024",
            type: "image/png",
            purpose: "maskable"
          }
        ]
      },
      workbox: {
        globPatterns: ["**/*.{js,css,html,ico,png,svg,woff2}"],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/api\.github\.com\/.*/i,
            handler: "NetworkFirst",
            options: {
              cacheName: "github-api-cache",
              expiration: {
                maxEntries: 50,
                maxAgeSeconds: 60 * 5 // 5 minutes
              }
            }
          },
          {
            urlPattern: /^https:\/\/.*\.githubusercontent\.com\/.*/i,
            handler: "CacheFirst",
            options: {
              cacheName: "github-content-cache",
              expiration: {
                maxEntries: 100,
                maxAgeSeconds: 60 * 60 * 24 // 24 hours
              }
            }
          }
        ]
      }
    })
  ],
  server: {
    port: 5173,
    strictPort: false,
    host: true
  },
  build: {
    target: "esnext"
  }
});

