import tailwindcss from "@tailwindcss/vite";
import vuetify, { transformAssetUrls } from "vite-plugin-vuetify";

export default defineNuxtConfig({
  compatibilityDate: "2026-03-02",
  extends: ["@white-rabbit/shared"],
  ssr: false,
  devServer: { port: 1420 },
  devtools: { enabled: false },
  build: {
    transpile: ["vuetify"],
  },
  css: ["~/assets/css/tailwind.css"],
  vite: {
    plugins: [tailwindcss(), vuetify({ autoImport: true })],
    vue: {
      template: {
        transformAssetUrls,
      },
    },
  },
});
