import { mount, type ComponentMountingOptions } from "@vue/test-utils";
import { computed } from "vue";
import type { Component } from "vue";
import type { ThemeDefinition } from "./themes/types";
import { registerTheme, clearThemes } from "./themes/registry";
import { tailwindDefault } from "./themes/tailwind-default";
import { md3Expressive } from "./themes/md3-expressive";
import { THEME_KEY, MODE_KEY } from "./themes/types";

/**
 * Mount a component inside a theme context.
 *
 * Uses @vue/test-utils `global.provide` to inject the theme and mode
 * directly, bypassing the need for createThemeRoot (which requires
 * being called inside a component setup).
 *
 * Pass `theme: "md3-expressive"` in options to mount under the MD3 theme.
 */
export function mountWithTheme<T extends Component>(
  component: T,
  options: ComponentMountingOptions<T> & { theme?: string } = {},
) {
  const { theme: themeName, ...mountOptions } = options;

  clearThemes();
  registerTheme(tailwindDefault);
  registerTheme(md3Expressive);
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.mode;

  const themeDef: ThemeDefinition =
    themeName === "md3-expressive" ? md3Expressive : tailwindDefault;
  const themeRef = computed(() => themeDef);
  const modeRef = computed<"light" | "dark">(() => "light");

  return mount(component, {
    ...mountOptions,
    global: {
      ...mountOptions.global,
      provide: {
        ...(mountOptions.global?.provide as
          | Record<symbol, unknown>
          | undefined),
        [THEME_KEY as symbol]: themeRef,
        [MODE_KEY as symbol]: modeRef,
      },
    },
  });
}
