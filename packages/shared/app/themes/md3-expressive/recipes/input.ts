import type { ComponentRecipe } from "../../types";

export const inputRecipe: ComponentRecipe = {
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
    "caret-primary",
  ],
  variants: {
    default: [],
    error: ["border-error", "text-error", "caret-error"],
  },
  sizes: {
    sm: ["h-10", "px-3", "text-sm"],
    md: ["h-14", "px-4", "text-base"],
    lg: ["h-16", "px-5", "text-lg"],
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
