import type { ComponentRecipe } from "../../types";

export const textareaRecipe: ComponentRecipe = {
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
    "resize-y",
  ],
  variants: {
    default: [],
    error: ["border-error", "text-error"],
  },
  sizes: {
    sm: ["px-2.5", "py-1.5", "text-sm"],
    md: ["px-3", "py-2", "text-sm"],
    lg: ["px-4", "py-3", "text-base"],
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
