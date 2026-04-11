import { describe, it, expect } from "vitest";
import { resolveRecipe } from "../../../composables/useRecipe";
import { md3Expressive } from "../index";

describe("md3-expressive recipes", () => {
  const { recipes } = md3Expressive;

  it("has all 13 component recipes", () => {
    const expected = [
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
    ];
    for (const name of expected) {
      expect(recipes[name], `missing recipe: ${name}`).toBeDefined();
    }
  });

  describe("button recipe", () => {
    const recipe = recipes.button!;

    it("includes md3-state-layer in base classes", () => {
      expect(recipe.base).toContain("md3-state-layer");
    });

    it("uses rounded-full for MD3 expressive shape", () => {
      expect(recipe.base).toContain("rounded-full");
    });

    it("resolves solid variant with elevation", () => {
      const classes = resolveRecipe(recipe, { variant: "solid", size: "md" });
      expect(classes).toContain("bg-primary");
      expect(classes).toContain("text-on-primary");
      expect(classes).toContain("shadow-[var(--wr-elevation-1)]");
    });

    it("resolves outlined variant without elevation", () => {
      const classes = resolveRecipe(recipe, {
        variant: "outlined",
        size: "md",
      });
      expect(classes).toContain("border");
      expect(classes).toContain("shadow-none");
    });

    it("uses state layer hover instead of brightness", () => {
      const classes = resolveRecipe(recipe, { variant: "solid", size: "md" });
      expect(classes).toContain("hover:before:opacity-[0.08]");
      expect(classes).not.toContain("hover:brightness-110");
    });

    it("applies MD3 disabled opacity", () => {
      const classes = resolveRecipe(recipe, {
        variant: "solid",
        size: "md",
        disabled: true,
      });
      expect(classes).toContain("opacity-[0.38]");
    });
  });

  describe("input recipe", () => {
    const recipe = recipes.input!;

    it("uses rounded-xl for MD3 shape", () => {
      expect(recipe.base).toContain("rounded-xl");
    });

    it("uses surface-container background", () => {
      expect(recipe.base).toContain("bg-surface-container");
    });

    it("has larger MD3 sizes", () => {
      expect(recipe.sizes.md).toContain("h-14");
    });
  });

  describe("card recipe", () => {
    const recipe = recipes.card!;

    it("uses rounded-xl", () => {
      expect(recipe.base).toContain("rounded-xl");
    });

    it("elevated variant uses MD3 elevation token", () => {
      expect(recipe.variants.elevated).toContain(
        "shadow-[var(--wr-elevation-1)]",
      );
    });

    it("elevated variant uses surface-container-low", () => {
      expect(recipe.variants.elevated).toContain("bg-surface-container-low");
    });
  });

  describe("chip recipe", () => {
    const recipe = recipes.chip!;

    it("includes md3-state-layer", () => {
      expect(recipe.base).toContain("md3-state-layer");
    });

    it("tonal variant uses surface-container-high", () => {
      expect(recipe.variants.tonal).toContain("bg-surface-container-high");
    });
  });

  describe("dialog recipe", () => {
    const recipe = recipes.dialog!;

    it("content uses extra-large rounding", () => {
      const classes = resolveRecipe(recipe, { variant: "content", size: "md" });
      expect(classes).toContain("rounded-[1.75rem]");
    });

    it("content uses surface-container-high", () => {
      expect(recipe.variants.content).toContain("bg-surface-container-high");
    });
  });

  describe("tooltip recipe", () => {
    const recipe = recipes.tooltip!;

    it("uses inverse-surface colors", () => {
      expect(recipe.variants.content).toContain("bg-inverse-surface");
      expect(recipe.variants.content).toContain("text-inverse-on-surface");
    });
  });

  describe("dataTable recipe", () => {
    const recipe = recipes.dataTable!;

    it("row does not use md3-state-layer (incompatible with <tr>)", () => {
      expect(recipe.variants.row).not.toContain("md3-state-layer");
    });

    it("uses subtle background-based hover", () => {
      expect(recipe.interactions.hover).toContain("hover:bg-on-surface/[0.04]");
    });
  });

  describe("icon recipe", () => {
    const recipe = recipes.icon!;

    it("has larger sizes than tailwind-default", () => {
      // MD3 uses size-6 for md vs tailwind's size-5
      expect(recipe.sizes.md).toContain("size-6");
    });
  });
});
