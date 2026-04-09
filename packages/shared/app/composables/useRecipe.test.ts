import { describe, it, expect } from "vitest";
import type { ComponentRecipe } from "../themes/types";
import { resolveRecipe } from "./useRecipe";

const recipe: ComponentRecipe = {
  base: ["inline-flex", "items-center"],
  variants: {
    solid: ["bg-primary", "text-on-primary"],
    outlined: ["border", "border-outline"],
    ghost: ["bg-transparent"],
  },
  sizes: {
    sm: ["h-8", "px-3", "text-sm"],
    md: ["h-10", "px-4"],
    lg: ["h-12", "px-6", "text-lg"],
  },
  interactions: {
    hover: ["hover:brightness-110"],
    focus: ["focus-visible:outline-2"],
    pressed: ["active:scale-[0.98]"],
    disabled: ["opacity-50", "pointer-events-none"],
  },
  compound: [
    { variant: "solid", size: "lg", classes: ["font-bold"] },
    { size: "sm", classes: ["gap-1"] },
  ],
};

describe("resolveRecipe", () => {
  it("returns base classes with no props", () => {
    const result = resolveRecipe(recipe, {});
    expect(result).toContain("inline-flex");
    expect(result).toContain("items-center");
  });

  it("includes variant classes", () => {
    const result = resolveRecipe(recipe, { variant: "solid" });
    expect(result).toContain("bg-primary");
    expect(result).toContain("text-on-primary");
  });

  it("includes size classes", () => {
    const result = resolveRecipe(recipe, { size: "lg" });
    expect(result).toContain("h-12");
    expect(result).toContain("text-lg");
  });

  it("includes hover/focus/pressed when not disabled", () => {
    const result = resolveRecipe(recipe, { variant: "solid", size: "md" });
    expect(result).toContain("hover:brightness-110");
    expect(result).toContain("focus-visible:outline-2");
    expect(result).toContain("active:scale-[0.98]");
    expect(result).not.toContain("opacity-50");
  });

  it("replaces interaction classes with disabled when disabled", () => {
    const result = resolveRecipe(recipe, {
      variant: "solid",
      size: "md",
      disabled: true,
    });
    expect(result).toContain("opacity-50");
    expect(result).toContain("pointer-events-none");
    // Hover/focus/pressed must NOT be present
    expect(result).not.toContain("hover:brightness-110");
    expect(result).not.toContain("focus-visible:outline-2");
    expect(result).not.toContain("active:scale-[0.98]");
  });

  it("resolves compound variants matching variant + size", () => {
    const result = resolveRecipe(recipe, { variant: "solid", size: "lg" });
    expect(result).toContain("font-bold");
  });

  it("resolves compound variants matching size only", () => {
    const result = resolveRecipe(recipe, { variant: "ghost", size: "sm" });
    expect(result).toContain("gap-1");
    expect(result).not.toContain("font-bold");
  });

  it("does not include compound when no match", () => {
    const result = resolveRecipe(recipe, { variant: "outlined", size: "md" });
    expect(result).not.toContain("font-bold");
    expect(result).not.toContain("gap-1");
  });

  it("ignores unknown variant gracefully", () => {
    const result = resolveRecipe(recipe, { variant: "nonexistent" });
    // Should still have base + interactions, no variant classes
    expect(result).toContain("inline-flex");
    expect(result).toContain("hover:brightness-110");
    expect(result).not.toContain("bg-primary");
  });

  it("handles recipe with no compound field", () => {
    const minimal: ComponentRecipe = {
      base: ["btn"],
      variants: {},
      sizes: {},
      interactions: {},
    };
    const result = resolveRecipe(minimal, { variant: "solid", size: "md" });
    expect(result).toEqual(["btn"]);
  });

  it("handles recipe with dragged interaction", () => {
    const withDrag: ComponentRecipe = {
      base: [],
      variants: {},
      sizes: {},
      interactions: { dragged: ["opacity-70", "scale-105"] },
    };
    const result = resolveRecipe(withDrag, {});
    expect(result).toContain("opacity-70");
    expect(result).toContain("scale-105");
  });
});
