import { createThemeRoot } from "../composables/useTheme";
import { useMode } from "../composables/useMode";

/**
 * Nuxt plugin: initialises the theme system.
 *
 * Reads persisted mode from localStorage, provides theme context to the
 * entire component tree, and syncs data attributes on <html>.
 *
 * The default theme name is configurable via runtime config; falls back
 * to "tailwind-default".
 */
export default defineNuxtPlugin(() => {
  const { mode } = useMode();

  // TODO: make default theme configurable via runtimeConfig when
  // multiple themes are available (Phase 4).
  const themeName = "tailwind-default";

  createThemeRoot({ theme: themeName, mode });
});
