import type { ComponentRecipe } from "../../types";

export const inputRecipe: ComponentRecipe = {
  base: [
    "w-full",
    "rounded-md",
    "border",
    "border-outline-variant",
    "bg-surface",
    "text-on-surface",
    "placeholder:text-on-surface-variant",
    "transition-colors",
    "duration-normal",
    "ease-standard",
  ],
  variants: {
    default: [],
    error: ["border-error", "text-error"],
  },
  sizes: {
    sm: ["h-8", "px-2.5", "text-sm"],
    md: ["h-10", "px-3", "text-sm"],
    lg: ["h-12", "px-4", "text-base"],
  },
  interactions: {
    hover: ["hover:border-outline"],
    focus: [
      "focus:outline-2",
      "focus:outline-offset-0",
      "focus:outline-primary",
      "focus:border-primary",
    ],
    disabled: ["opacity-50", "cursor-not-allowed", "bg-surface-variant"],
  },
};
