import type { ThemeDefinition } from "../types";
import { buttonRecipe } from "./recipes/button";

export const tailwindDefault: ThemeDefinition = {
  name: "tailwind-default",
  recipes: {
    button: buttonRecipe,
  },
  stylesheets: ["./tokens.css"],
};
