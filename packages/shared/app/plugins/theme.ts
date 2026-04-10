import { createThemeRoot } from "../composables/useTheme";
import { useMode } from "../composables/useMode";
import { registerTheme } from "../themes/registry";
import { tailwindDefault } from "../themes/tailwind-default";

/**
 * Nuxt plugin: initialises the theme system.
 *
 * Registers all available themes, reads persisted mode from localStorage,
 * provides theme context to the entire component tree, and syncs data
 * attributes on <html>.
 */
export default defineNuxtPlugin(() => {
  registerTheme(tailwindDefault);

  const { mode } = useMode();

  // TODO: make default theme configurable via runtimeConfig when
  // multiple themes are available (Phase 4).
  createThemeRoot({ theme: "tailwind-default", mode });
});
