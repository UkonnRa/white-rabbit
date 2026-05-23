export default defineNuxtConfig({
  compatibilityDate: "2026-03-02",
  extends: ["@white-rabbit/shared"],
  ssr: false,
  devServer: { port: 1420 },
  devtools: { enabled: false },
  css: ["~/assets/css/tailwind.css"],
  modules: ["@nuxt/ui"],
  ui: {
    fonts: false,
  },
  icon: {
    serverBundle: false,
  },
});
