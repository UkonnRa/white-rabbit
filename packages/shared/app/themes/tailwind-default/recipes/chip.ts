import type { ComponentRecipe } from "../../types";

export const chipRecipe: ComponentRecipe = {
  base: [
    "inline-flex",
    "items-center",
    "gap-1",
    "rounded-full",
    "font-medium",
    "whitespace-nowrap",
    "transition-colors",
    "duration-normal",
    "ease-standard",
    "select-none",
  ],
  variants: {
    solid: ["bg-primary", "text-on-primary"],
    outlined: ["border", "border-outline", "text-on-surface"],
    tonal: ["bg-surface-variant", "text-on-surface-variant"],
  },
  sizes: {
    sm: ["h-6", "px-2", "text-xs"],
    md: ["h-7", "px-3", "text-sm"],
    lg: ["h-8", "px-4", "text-sm"],
  },
  interactions: {
    hover: ["hover:brightness-110"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-1",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-50", "pointer-events-none"],
  },
};
