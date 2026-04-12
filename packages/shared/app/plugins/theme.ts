import { computed, toValue, watchEffect } from "vue";
import type { InjectionKey, Ref } from "vue";
import { useLocalStorage } from "@vueuse/core";
import { useMode } from "../composables/useMode";
import { registerTheme } from "../themes/registry";
import { getTheme } from "../themes/registry";
import { tailwindDefault } from "../themes/tailwind-default";
import { md3Expressive } from "../themes/md3-expressive";
import {
  applySeedTokensToElement as applyMD3Seed,
  clearSeedTokensFromElement as clearMD3Seed,
} from "../themes/md3-expressive/seed";
import {
  applyTailwindTokensToElement,
  clearTailwindTokensFromElement,
} from "../themes/tailwind-default/seed";
import { THEME_KEY, MODE_KEY } from "../themes/types";

/** Injection key for the active theme name — used by the theme switcher UI. */
export const THEME_NAME_KEY = Symbol("wr-theme-name") as InjectionKey<
  Ref<string>
>;

/** Injection key for the seed color — used by the seed color picker UI. */
export const SEED_COLOR_KEY = Symbol("wr-seed-color") as InjectionKey<
  Ref<string>
>;

/**
 * Nuxt plugin: initialises the multi-theme system.
 *
 * Registers all available themes, reads persisted theme name, seed color,
 * and mode from localStorage, provides theme context to the entire
 * component tree via app-level provide, and syncs data attributes +
 * MD3 dynamic tokens on <html>.
 */
export default defineNuxtPlugin((nuxtApp) => {
  registerTheme(tailwindDefault);
  registerTheme(md3Expressive);

  const themeName = useLocalStorage("app-theme", "tailwind-default");
  const seedColor = useLocalStorage("app-seed-color", "#6750A4");
  const { mode } = useMode();

  const themeDef = computed(() => getTheme(themeName.value));
  const modeRef = computed(() => toValue(mode));

  // Sync data attributes on <html> and apply per-theme seed tokens.
  // Each theme has its own seed strategy:
  //   - md3-expressive: SchemeTonalSpot derives all 29+ roles from seed
  //   - tailwind-default: seed picks accent hue; surfaces stay neutral
  watchEffect(() => {
    if (typeof document !== "undefined") {
      const html = document.documentElement;
      const dark = modeRef.value === "dark";
      html.dataset.theme = themeName.value;
      html.dataset.mode = modeRef.value;

      if (themeName.value === "md3-expressive") {
        clearTailwindTokensFromElement(html);
        applyMD3Seed(html, seedColor.value, dark);
      } else {
        clearMD3Seed(html);
        applyTailwindTokensToElement(html, seedColor.value, dark);
      }
    }
  });

  // Register theme directives (e.g. v-ripple for MD3)
  watchEffect(() => {
    const directives = themeDef.value.directives;
    if (directives) {
      for (const [name, directive] of Object.entries(directives)) {
        nuxtApp.vueApp.directive(name, directive);
      }
    }
  });

  // App-level provide — visible to all components
  nuxtApp.vueApp.provide(THEME_KEY, themeDef);
  nuxtApp.vueApp.provide(MODE_KEY, modeRef);
  nuxtApp.vueApp.provide(THEME_NAME_KEY, themeName);
  nuxtApp.vueApp.provide(SEED_COLOR_KEY, seedColor);
});
