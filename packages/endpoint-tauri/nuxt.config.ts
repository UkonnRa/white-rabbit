export default defineNuxtConfig({
  compatibilityDate: "2026-03-02",
  extends: ["@white-rabbit/shared"],
  modules: ["@nuxt/ui"],
  ssr: false,
  devServer: { port: 1420 },
  devtools: { enabled: false },
});
