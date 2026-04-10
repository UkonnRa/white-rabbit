import { describe, it, expect, beforeEach } from "vitest";
import { registerTheme, clearThemes } from "../../themes/registry";
import { tailwindDefault } from "../../themes/tailwind-default";
import { createThemeRoot } from "../../composables/useTheme";
import { withNestedSetup } from "../../test-utils";

// We can't mount .vue SFCs directly without a Vue compiler plugin in vitest,
// so we test the integration by verifying useRecipe resolves the correct
// classes for button within the tailwind-default theme context.

import { useRecipe } from "../../composables/useRecipe";

beforeEach(() => {
  clearThemes();
  registerTheme(tailwindDefault);
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.mode;
});

describe("AppButton integration (theme -> recipe -> classes)", () => {
  it("resolves solid/md button classes from tailwind-default theme", () => {
    const { childResult, unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
      () => useRecipe("button", { variant: "solid", size: "md" }),
    );

    const classes = childResult.value;
    expect(classes).toContain("inline-flex");
    expect(classes).toContain("bg-primary");
    expect(classes).toContain("text-on-primary");
    expect(classes).toContain("h-10");
    expect(classes).toContain("px-4");
    expect(classes).toContain("hover:brightness-110");
    unmount();
  });

  it("resolves outlined/sm button classes", () => {
    const { childResult, unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
      () => useRecipe("button", { variant: "outlined", size: "sm" }),
    );

    const classes = childResult.value;
    expect(classes).toContain("border");
    expect(classes).toContain("border-outline");
    expect(classes).toContain("h-8");
    expect(classes).toContain("px-3");
    unmount();
  });

  it("resolves disabled button without interaction classes", () => {
    const { childResult, unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
      () =>
        useRecipe("button", { variant: "solid", size: "md", disabled: true }),
    );

    const classes = childResult.value;
    expect(classes).toContain("opacity-50");
    expect(classes).toContain("pointer-events-none");
    expect(classes).not.toContain("hover:brightness-110");
    unmount();
  });

  it("sets data-theme and data-mode on <html>", () => {
    const { unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "dark" }),
      () => useRecipe("button", { variant: "solid", size: "md" }),
    );

    expect(document.documentElement.dataset.theme).toBe("tailwind-default");
    expect(document.documentElement.dataset.mode).toBe("dark");
    unmount();
  });

  it("returns empty array for unregistered component recipe", () => {
    const { childResult, unmount } = withNestedSetup(
      () => createThemeRoot({ theme: "tailwind-default", mode: "light" }),
      () => useRecipe("dialog", {}),
    );

    expect(childResult.value).toEqual([]);
    unmount();
  });
});
