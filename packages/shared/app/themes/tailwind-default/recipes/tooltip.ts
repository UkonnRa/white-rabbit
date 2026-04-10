import type { ComponentRecipe } from "../../types";

export const tooltipRecipe: ComponentRecipe = {
  base: [],
  variants: {
    content: [
      "rounded-md",
      "bg-on-surface",
      "text-surface",
      "px-3",
      "py-1.5",
      "text-xs",
      "font-medium",
      "shadow-md",
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
