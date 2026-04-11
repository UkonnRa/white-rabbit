import type { ComponentRecipe } from "../../types";

export const cardRecipe: ComponentRecipe = {
  base: [
    "rounded-xl",
    "text-on-surface",
    "transition-shadow",
    "duration-normal",
    "ease-standard",
  ],
  variants: {
    elevated: ["bg-surface-container-low", "shadow-[var(--wr-elevation-1)]"],
    outlined: ["bg-surface", "border", "border-outline-variant"],
    filled: ["bg-surface-container-highest"],
  },
  sizes: {
    sm: ["p-3"],
    md: ["p-4"],
    lg: ["p-6"],
  },
  interactions: {
    hover: ["hover:shadow-[var(--wr-elevation-2)]"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-2",
      "focus-visible:outline-primary",
    ],
  },
};
