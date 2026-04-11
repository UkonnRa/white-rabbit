import type { ComponentRecipe } from "../../types";

export const textareaRecipe: ComponentRecipe = {
  base: [
    "w-full",
    "rounded-xl",
    "border",
    "border-outline-variant",
    "bg-surface-container",
    "text-on-surface",
    "placeholder:text-on-surface-variant",
    "transition-colors",
    "duration-normal",
    "ease-standard",
    "resize-y",
    "caret-primary",
  ],
  variants: {
    default: [],
    error: ["border-error", "text-error", "caret-error"],
  },
  sizes: {
    sm: ["px-3", "py-2", "text-sm"],
    md: ["px-4", "py-3", "text-base"],
    lg: ["px-5", "py-4", "text-lg"],
  },
  interactions: {
    hover: ["hover:border-on-surface"],
    focus: [
      "focus:outline-2",
      "focus:outline-offset-0",
      "focus:outline-primary",
      "focus:border-primary",
    ],
    disabled: [
      "opacity-[0.38]",
      "cursor-not-allowed",
      "bg-surface-container-low",
    ],
  },
};
