import type { ComponentRecipe } from "../../types";

export const tooltipRecipe: ComponentRecipe = {
  base: [],
  variants: {
    content: [
      "rounded-lg",
      "bg-inverse-surface",
      "text-inverse-on-surface",
      "px-4",
      "py-2",
      "text-sm",
      "shadow-[var(--wr-elevation-2)]",
      "data-[state=delayed-open]:animate-in",
      "data-[state=delayed-open]:fade-in-0",
      "data-[state=delayed-open]:zoom-in-95",
      "data-[state=closed]:animate-out",
      "data-[state=closed]:fade-out-0",
      "data-[state=closed]:zoom-out-95",
    ],
  },
  sizes: {},
  interactions: {},
};
