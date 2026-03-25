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
    plugins: [
      tailwindcss(),
      // @ts-expect-error vite-plugin-vuetify type mismatch with nuxt vite plugin array
      vuetify({ autoImport: true }),
    ],
    vue: {
      template: {
        transformAssetUrls,
      },
    },
  },
});
