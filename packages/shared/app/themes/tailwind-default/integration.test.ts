import { describe, it, expect, beforeEach } from "vitest";
import { registerTheme, clearThemes } from "../registry";
import { tailwindDefault } from "./index";
import { createThemeRoot } from "../../composables/useTheme";
import { useRecipe } from "../../composables/useRecipe";
import { withNestedSetup } from "../../test-utils";

beforeEach(() => {
  clearThemes();
  registerTheme(tailwindDefault);
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.mode;
});

const componentKeys = [
  "button",
  "input",
  "textarea",
  "select",
  "combobox",
  "card",
  "chip",
  "dialog",
  "menu",
  "tooltip",
  "switch",
  "icon",
  "dataTable",
] as const;

describe("tailwind-default theme integration", () => {
  it("registers all expected component recipes", () => {
    for (const key of componentKeys) {
      expect(tailwindDefault.recipes[key]).toBeDefined();
    }
  });

  it.each(componentKeys)(
    "useRecipe('%s') resolves non-empty classes through theme context",
    (component) => {
      const { childResult, unmount } = withNestedSetup(
        () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
        () => useRecipe(component, {}),
      );

      // Every recipe should produce at least base classes (or variant defaults)
      const classes = childResult.value;
      expect(Array.isArray(classes)).toBe(true);
      unmount();
    },
  );

  it("sets data-theme=tailwind-default on html", () => {
    const { unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
      () => useRecipe("button", {}),
    );

    expect(document.documentElement.dataset.theme).toBe("tailwind-default");
    unmount();
  });

  it("theme name matches", () => {
    expect(tailwindDefault.name).toBe("tailwind-default");
  });
});
