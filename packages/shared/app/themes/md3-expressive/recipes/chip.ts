import type { ComponentRecipe } from "../../types";

export const chipRecipe: ComponentRecipe = {
  base: [
    "md3-state-layer",
    "inline-flex",
    "items-center",
    "gap-1",
    "rounded-full",
    "font-medium",
    "whitespace-nowrap",
    "transition-all",
    "duration-normal",
    "ease-standard",
    "select-none",
  ],
  variants: {
    solid: ["bg-primary", "text-on-primary"],
    outlined: ["border", "border-outline", "text-on-surface"],
    tonal: ["bg-surface-container-high", "text-on-surface-variant"],
  },
  sizes: {
    sm: ["h-7", "px-3", "text-xs"],
    md: ["h-8", "px-4", "text-sm"],
    lg: ["h-9", "px-5", "text-sm"],
  },
  interactions: {
    hover: ["hover:before:opacity-[0.08]"],
    focus: [
      "focus-visible:before:opacity-[0.10]",
      "focus-visible:outline-2",
      "focus-visible:outline-offset-1",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-[0.38]", "pointer-events-none"],
  },
};
