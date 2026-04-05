import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        id: "/",
        name: "Tastebase",
        short_name: "Tastebase",
        description:
          "Track tastings with photo and voice capture, scoring, and tasting notes.",
        theme_color: "#d63831",
        background_color: "#1a1a2e",
        display: "standalone",
        scope: "/",
        start_url: "/",
        icons: [
          {
            src: "pwa-192x192.png",
            sizes: "192x192",
            type: "image/png",
          },
          {
            src: "pwa-512x512.png",
            sizes: "512x512",
            type: "image/png",
          },
          {
            src: "pwa-512x512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "maskable",
          },
        ],
      },
      workbox: {
        globPatterns: ["**/*.{js,css,html,ico,png,svg,woff2}"],
        globIgnores: ["config.js"],
        runtimeCaching: [
          {
            // Runtime config — excluded from precache (build has empty values),
            // cached at runtime so it works offline
            urlPattern: /\/config\.js$/,
            handler: "NetworkFirst",
            options: {
              cacheName: "runtime-config",
            },
          },
          {
            urlPattern:
              /^https:\/\/api\.tastebase\.ahara\.io\/(?:recipes|tastings)/i,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-cache",
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            // Media images from S3
            urlPattern: /\.(?:png|jpg|jpeg|webp|gif)$/i,
            handler: "CacheFirst",
            options: {
              cacheName: "image-cache",
              expiration: { maxEntries: 200 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
        ],
      },
    }),
  ],
});
