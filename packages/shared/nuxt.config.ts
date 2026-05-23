export default defineNuxtConfig({
  modules: ["@nuxt/eslint", "@vueuse/nuxt", "@nuxt/ui"],
  css: ["~/assets/css/tailwind.css"],
  ui: {
    fonts: false,
  },
  icon: {
    serverBundle: false,
  },
});
