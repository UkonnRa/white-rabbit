import type { ComponentRecipe } from "../../types";

export const cardRecipe: ComponentRecipe = {
  base: [
    "rounded-lg",
    "bg-surface",
    "text-on-surface",
    "transition-shadow",
    "duration-normal",
    "ease-standard",
  ],
  variants: {
    elevated: ["shadow-md"],
    outlined: ["border", "border-outline-variant"],
    filled: ["bg-surface-variant"],
  },
  sizes: {
    sm: ["p-3"],
    md: ["p-4"],
    lg: ["p-6"],
  },
  interactions: {
    hover: ["hover:shadow-lg"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-2",
      "focus-visible:outline-primary",
    ],
  },
};
