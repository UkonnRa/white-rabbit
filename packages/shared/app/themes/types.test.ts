import { describe, it, expectTypeOf } from "vitest";
import type {
  BaseTokens,
  MD3Tokens,
  ComponentRecipe,
  ThemeDefinition,
} from "./types";

describe("theme type contracts", () => {
  it("BaseTokens requires all color roles", () => {
    expectTypeOf<BaseTokens["color"]>().toHaveProperty("primary");
    expectTypeOf<BaseTokens["color"]>().toHaveProperty("onPrimary");
    expectTypeOf<BaseTokens["color"]>().toHaveProperty("error");
    expectTypeOf<BaseTokens["color"]>().toHaveProperty("surface");
    expectTypeOf<BaseTokens["color"]>().toHaveProperty("outline");
  });

  it("MD3Tokens extends BaseTokens with additional color roles", () => {
    expectTypeOf<MD3Tokens["color"]>().toHaveProperty("tertiary");
    expectTypeOf<MD3Tokens["color"]>().toHaveProperty("primaryContainer");
    expectTypeOf<MD3Tokens["color"]>().toHaveProperty("inverseSurface");
    // Still has base roles
    expectTypeOf<MD3Tokens["color"]>().toHaveProperty("primary");
  });

  it("MD3Tokens has stateLayer and elevation", () => {
    expectTypeOf<MD3Tokens>().toHaveProperty("stateLayer");
    expectTypeOf<MD3Tokens>().toHaveProperty("elevation");
    expectTypeOf<MD3Tokens["stateLayer"]>().toHaveProperty("hoverOpacity");
    expectTypeOf<MD3Tokens["elevation"]>().toHaveProperty("level0");
  });

  it("ComponentRecipe interactions are optional per-state", () => {
    const recipe: ComponentRecipe = {
      base: [],
      variants: {},
      sizes: {},
      interactions: {},
    };
    expectTypeOf(recipe.interactions.hover).toEqualTypeOf<
      string[] | undefined
    >();
    expectTypeOf(recipe.interactions.disabled).toEqualTypeOf<
      string[] | undefined
    >();
  });

  it("ThemeDefinition satisfies structural contract", () => {
    const theme: ThemeDefinition = {
      name: "test",
      recipes: {
        button: {
          base: ["btn"],
          variants: { solid: ["bg-primary"] },
          sizes: { md: ["h-10"] },
          interactions: {},
        },
      },
    };
    expectTypeOf(theme).toMatchTypeOf<ThemeDefinition>();
    expectTypeOf(theme.directives).toEqualTypeOf<
      Record<string, import("vue").Directive> | undefined
    >();
  });
});
