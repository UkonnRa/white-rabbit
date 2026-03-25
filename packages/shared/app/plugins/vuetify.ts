import { buildTheme, DEFAULT_SEED } from "~/composables/useAppTheme";
import { createVuetify } from "vuetify";

// Restore persisted seed so initial render matches the saved color
const seed =
  (typeof localStorage !== "undefined" &&
    localStorage.getItem("app-seed-color")) ||
  DEFAULT_SEED;

export default defineNuxtPlugin((app) => {
  const vuetify = createVuetify({
    theme: {
      defaultTheme: "light",
      themes: {
        light: buildTheme(seed, false),
        dark: buildTheme(seed, true),
      },
    },
    icons: {
      defaultSet: "mdi",
    },
  });
  app.vueApp.use(vuetify);
});
