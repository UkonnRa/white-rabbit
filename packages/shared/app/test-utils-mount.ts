import { mount, type ComponentMountingOptions } from "@vue/test-utils";
import { computed } from "vue";
import type { Component } from "vue";
import { registerTheme, clearThemes } from "./themes/registry";
import { tailwindDefault } from "./themes/tailwind-default";
import { THEME_KEY, MODE_KEY } from "./themes/types";

/**
 * Mount a component inside a theme context.
 *
 * Uses @vue/test-utils `global.provide` to inject the theme and mode
 * directly, bypassing the need for createThemeRoot (which requires
 * being called inside a component setup).
 */
export function mountWithTheme<T extends Component>(
  component: T,
  options: ComponentMountingOptions<T> = {},
) {
  clearThemes();
  registerTheme(tailwindDefault);
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.mode;

  const themeRef = computed(() => tailwindDefault);
  const modeRef = computed<"light" | "dark">(() => "light");

  return mount(component, {
    ...options,
    global: {
      ...options.global,
      provide: {
        ...(options.global?.provide as Record<symbol, unknown> | undefined),
        [THEME_KEY as symbol]: themeRef,
        [MODE_KEY as symbol]: modeRef,
      },
    },
  });
}
