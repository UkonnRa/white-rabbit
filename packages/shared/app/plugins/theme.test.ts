import { describe, it, expect, beforeEach } from "vitest";
import { createApp, defineComponent, h, inject } from "vue";
import type { ComputedRef } from "vue";
import { clearThemes, getTheme, listThemes } from "../themes/registry";
import { registerTheme } from "../themes/registry";
import { tailwindDefault } from "../themes/tailwind-default";
import { md3Expressive } from "../themes/md3-expressive";
import type { ThemeDefinition } from "../themes/types";
import { THEME_KEY, MODE_KEY } from "../themes/types";
import { computed, toValue, watchEffect } from "vue";
import { useMode, _resetMode } from "../composables/useMode";

/**
 * Simulates what the Nuxt plugin does: app-level provide.
 *
 * This ensures that components using useTheme/useRecipe can resolve
 * the theme context when it's provided at the app level (not inside
 * a component setup).
 */
function installThemePlugin(app: ReturnType<typeof createApp>) {
  clearThemes();
  registerTheme(tailwindDefault);

  const { mode } = useMode();
  const themeDef = computed(() => getTheme("tailwind-default"));
  const modeRef = computed(() => toValue(mode));

  watchEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset.theme = "tailwind-default";
      document.documentElement.dataset.mode = modeRef.value;
    }
  });

  app.provide(THEME_KEY, themeDef);
  app.provide(MODE_KEY, modeRef);
}

beforeEach(() => {
  clearThemes();
  _resetMode("light");
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.mode;
});

describe("theme plugin (app-level provide)", () => {
  it("registers tailwind-default theme", () => {
    const app = createApp({ render: () => h("div") });
    installThemePlugin(app);
    expect(listThemes()).toContain("tailwind-default");
    app.unmount();
  });

  it("provides THEME_KEY to child components", () => {
    let injectedTheme: unknown = null;

    const Child = defineComponent({
      setup() {
        injectedTheme = inject(THEME_KEY);
        return () => h("div");
      },
    });

    const app = createApp(Child);
    installThemePlugin(app);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    expect(injectedTheme).toBeTruthy();
    expect((injectedTheme as ComputedRef<ThemeDefinition>).value.name).toBe(
      "tailwind-default",
    );

    app.unmount();
    root.remove();
  });

  it("provides MODE_KEY to child components", () => {
    let injectedMode: unknown = null;

    const Child = defineComponent({
      setup() {
        injectedMode = inject(MODE_KEY);
        return () => h("div");
      },
    });

    const app = createApp(Child);
    installThemePlugin(app);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    expect(injectedMode).toBeTruthy();
    expect((injectedMode as ComputedRef<string>).value).toBe("light");

    app.unmount();
    root.remove();
  });

  it("sets data-theme and data-mode on <html>", () => {
    const app = createApp({ render: () => h("div") });
    installThemePlugin(app);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    expect(document.documentElement.dataset.theme).toBe("tailwind-default");
    expect(document.documentElement.dataset.mode).toBe("light");

    app.unmount();
    root.remove();
  });

  it("child component using useRecipe works with app-level provide", () => {
    let classes: string[] = [];

    const Child = defineComponent({
      setup() {
        const theme = inject(THEME_KEY);
        if (theme) {
          const recipe = theme.value.recipes.button;
          if (recipe) {
            classes = [...recipe.base];
          }
        }
        return () => h("div");
      },
    });

    const app = createApp(Child);
    installThemePlugin(app);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    expect(classes).toContain("inline-flex");
    expect(classes).toContain("items-center");

    app.unmount();
    root.remove();
  });
});

describe("multi-theme support", () => {
  it("registers both tailwind-default and md3-expressive themes", () => {
    clearThemes();
    registerTheme(tailwindDefault);
    registerTheme(md3Expressive);
    expect(listThemes()).toContain("tailwind-default");
    expect(listThemes()).toContain("md3-expressive");
  });

  it("can switch theme and provide md3-expressive to components", () => {
    let injectedThemeName: string | null = null;

    const Child = defineComponent({
      setup() {
        const theme = inject(THEME_KEY);
        if (theme) {
          injectedThemeName = theme.value.name;
        }
        return () => h("div");
      },
    });

    clearThemes();
    registerTheme(tailwindDefault);
    registerTheme(md3Expressive);

    const themeDef = computed(() => getTheme("md3-expressive"));
    const modeRef = computed<"light" | "dark">(() => "light");

    const app = createApp(Child);
    app.provide(THEME_KEY, themeDef);
    app.provide(MODE_KEY, modeRef);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    expect(injectedThemeName).toBe("md3-expressive");

    app.unmount();
    root.remove();
  });

  it("md3-expressive button recipe uses state layer interaction", () => {
    clearThemes();
    registerTheme(md3Expressive);
    const theme = getTheme("md3-expressive");
    expect(theme.recipes.button).toBeDefined();
    expect(theme.recipes.button!.base).toContain("md3-state-layer");
  });

  it("md3-expressive has ripple directive registered", () => {
    expect(md3Expressive.directives).toBeDefined();
    expect(md3Expressive.directives!.ripple).toBeDefined();
  });

  it("component renders correctly under md3-expressive theme", () => {
    let buttonClasses: string[] = [];

    const Child = defineComponent({
      setup() {
        const theme = inject(THEME_KEY);
        if (theme) {
          const recipe = theme.value.recipes.button;
          if (recipe) {
            buttonClasses = [...recipe.base];
          }
        }
        return () => h("div");
      },
    });

    clearThemes();
    registerTheme(md3Expressive);

    const themeDef = computed(() => getTheme("md3-expressive"));
    const modeRef = computed<"light" | "dark">(() => "light");

    const app = createApp(Child);
    app.provide(THEME_KEY, themeDef);
    app.provide(MODE_KEY, modeRef);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);

    // MD3 button uses rounded-full + state layer
    expect(buttonClasses).toContain("rounded-full");
    expect(buttonClasses).toContain("md3-state-layer");
    // And doesn't use tailwind-default's rounded-md
    expect(buttonClasses).not.toContain("rounded-md");

    app.unmount();
    root.remove();
  });
});
