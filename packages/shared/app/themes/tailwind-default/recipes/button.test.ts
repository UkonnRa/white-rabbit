import { describe, it, expect } from "vitest";
import { resolveRecipe } from "../../../composables/useRecipe";
import { buttonRecipe } from "./button";

describe("tailwind-default button recipe", () => {
  it("base classes include layout and transition utilities", () => {
    const classes = resolveRecipe(buttonRecipe, {});
    expect(classes).toContain("inline-flex");
    expect(classes).toContain("items-center");
    expect(classes).toContain("rounded-md");
    expect(classes).toContain("font-medium");
  });

  it("solid variant has bg-primary and text-on-primary", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "solid" });
    expect(classes).toContain("bg-primary");
    expect(classes).toContain("text-on-primary");
    expect(classes).toContain("shadow-sm");
  });

  it("outlined variant has border and text-primary", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "outlined" });
    expect(classes).toContain("border");
    expect(classes).toContain("border-outline");
    expect(classes).toContain("text-primary");
  });

  it("ghost variant is transparent", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "ghost" });
    expect(classes).toContain("bg-transparent");
    expect(classes).toContain("text-on-surface");
  });

  it("text variant has text-primary only", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "text" });
    expect(classes).toContain("text-primary");
    expect(classes).toContain("bg-transparent");
  });

  it("sm size has correct height and padding", () => {
    const classes = resolveRecipe(buttonRecipe, { size: "sm" });
    expect(classes).toContain("h-8");
    expect(classes).toContain("px-3");
    expect(classes).toContain("text-sm");
  });

  it("md size has correct height and padding", () => {
    const classes = resolveRecipe(buttonRecipe, { size: "md" });
    expect(classes).toContain("h-10");
    expect(classes).toContain("px-4");
  });

  it("lg size has correct height and padding", () => {
    const classes = resolveRecipe(buttonRecipe, { size: "lg" });
    expect(classes).toContain("h-12");
    expect(classes).toContain("px-6");
    expect(classes).toContain("text-base");
  });

  it("includes hover/focus/pressed interactions when enabled", () => {
    const classes = resolveRecipe(buttonRecipe, {
      variant: "solid",
      size: "md",
    });
    expect(classes).toContain("hover:brightness-110");
    expect(classes).toContain("focus-visible:outline-2");
    expect(classes).toContain("focus-visible:outline-offset-2");
    expect(classes).toContain("focus-visible:outline-primary");
    expect(classes).toContain("active:scale-[0.98]");
  });

  it("disabled replaces interaction classes", () => {
    const classes = resolveRecipe(buttonRecipe, {
      variant: "solid",
      size: "md",
      disabled: true,
    });
    expect(classes).toContain("opacity-50");
    expect(classes).toContain("pointer-events-none");
    expect(classes).toContain("cursor-not-allowed");
    expect(classes).not.toContain("hover:brightness-110");
    expect(classes).not.toContain("active:scale-[0.98]");
  });

  it("ghost compound variant adds hover:bg-surface-variant", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "ghost" });
    expect(classes).toContain("hover:bg-surface-variant");
  });

  it("text compound variant adds hover:bg-surface-variant", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "text" });
    expect(classes).toContain("hover:bg-surface-variant");
  });

  it("solid variant does not get ghost compound classes", () => {
    const classes = resolveRecipe(buttonRecipe, { variant: "solid" });
    expect(classes).not.toContain("hover:bg-surface-variant");
  });

  it("all variant x size combinations produce non-empty class lists", () => {
    const variants = ["solid", "outlined", "ghost", "text"] as const;
    const sizes = ["sm", "md", "lg"] as const;
    for (const variant of variants) {
      for (const size of sizes) {
        const classes = resolveRecipe(buttonRecipe, { variant, size });
        expect(classes.length).toBeGreaterThan(0);
      }
    }
  });
});
