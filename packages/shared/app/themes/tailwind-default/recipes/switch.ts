import type { ComponentRecipe } from "../../types";

export const switchRecipe: ComponentRecipe = {
  base: [],
  variants: {
    root: [
      "relative",
      "inline-flex",
      "shrink-0",
      "cursor-pointer",
      "rounded-full",
      "border-2",
      "border-transparent",
      "bg-outline",
      "transition-colors",
      "duration-normal",
      "ease-standard",
      "data-[state=checked]:bg-primary",
    ],
    thumb: [
      "pointer-events-none",
      "block",
      "rounded-full",
      "bg-surface",
      "shadow-sm",
      "transition-transform",
      "duration-normal",
      "ease-standard",
    ],
  },
  sizes: {
    sm: ["h-5", "w-9"],
    md: ["h-6", "w-11"],
    lg: ["h-7", "w-[3.25rem]"],
  },
  interactions: {
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-2",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-50", "cursor-not-allowed"],
  },
};
