import { computed, provide, watchEffect, inject, toValue } from "vue";
import type { MaybeRef } from "vue";
import { THEME_KEY, MODE_KEY } from "../themes/types";
import { getTheme } from "../themes/registry";

/**
 * Root initialisation — call once in App.vue or a Nuxt plugin.
 *
 * - Sets `data-theme` / `data-mode` on `<html>`.
 * - Provides theme recipes and mode to the entire component tree.
 */
export function createThemeRoot(options: {
  theme: MaybeRef<string>;
  mode: MaybeRef<"light" | "dark">;
}) {
  const themeDef = computed(() => getTheme(toValue(options.theme)));
  const modeRef = computed(() => toValue(options.mode));

  // Apply data attributes to <html>
  watchEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset.theme = toValue(options.theme);
      document.documentElement.dataset.mode = modeRef.value;
    }
  });

  provide(THEME_KEY, themeDef);
  provide(MODE_KEY, modeRef);

  return { theme: themeDef, mode: modeRef };
}

/**
 * Consumer — use in any component to read the current theme.
 */
export function useTheme() {
  const theme = inject(THEME_KEY);
  const mode = inject(MODE_KEY);
  if (!theme || !mode) {
    throw new Error(
      "useTheme: no theme context found. Did you call createThemeRoot?",
    );
  }
  return { theme, mode };
}

/**
 * Subtree override — returns data attributes to bind on an existing element.
 *
 * Overrides provide/inject for descendants so child components resolve
 * recipes from the scoped theme.
 */
export function useThemeScope(options: {
  theme: MaybeRef<string>;
  mode: MaybeRef<"light" | "dark">;
}) {
  const themeDef = computed(() => getTheme(toValue(options.theme)));
  const modeRef = computed(() => toValue(options.mode));

  provide(THEME_KEY, themeDef);
  provide(MODE_KEY, modeRef);

  const themeAttrs = computed(() => ({
    "data-theme": toValue(options.theme),
    "data-mode": modeRef.value,
  }));

  return { themeAttrs, theme: themeDef, mode: modeRef };
}
