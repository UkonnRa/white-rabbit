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
      "border-outline",
      "bg-surface-container-highest",
      "transition-colors",
      "duration-normal",
      "ease-emphasized",
      "data-[state=checked]:bg-primary",
      "data-[state=checked]:border-primary",
    ],
    thumb: [
      "pointer-events-none",
      "block",
      "rounded-full",
      "bg-outline",
      "shadow-[var(--wr-elevation-1)]",
      "transition-all",
      "duration-normal",
      "ease-emphasized",
      "data-[state=checked]:bg-on-primary",
    ],
  },
  sizes: {
    sm: ["h-6", "w-11"],
    md: ["h-8", "w-[3.25rem]"],
    lg: ["h-9", "w-[3.75rem]"],
  },
  interactions: {
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-2",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-[0.38]", "cursor-not-allowed"],
  },
};
