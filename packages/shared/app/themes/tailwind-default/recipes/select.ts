import type { ComponentRecipe } from "../../types";

export const selectRecipe: ComponentRecipe = {
  base: [],
  variants: {
    trigger: [
      "inline-flex",
      "items-center",
      "justify-between",
      "gap-2",
      "rounded-md",
      "border",
      "border-outline-variant",
      "bg-surface",
      "text-on-surface",
      "transition-colors",
      "duration-normal",
      "ease-standard",
      "cursor-pointer",
    ],
    content: [
      "overflow-hidden",
      "rounded-md",
      "border",
      "border-outline-variant",
      "bg-surface",
      "text-on-surface",
      "shadow-lg",
    ],
    item: [
      "relative",
      "flex",
      "items-center",
      "rounded-sm",
      "text-sm",
      "cursor-pointer",
      "select-none",
      "outline-none",
    ],
    itemIndicator: [
      "absolute",
      "left-2",
      "inline-flex",
      "items-center",
      "justify-center",
    ],
  },
  sizes: {
    sm: ["h-8", "px-2.5", "text-sm"],
    md: ["h-10", "px-3", "text-sm"],
    lg: ["h-12", "px-4", "text-base"],
  },
  interactions: {
    hover: ["hover:border-outline"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-0",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-50", "pointer-events-none"],
  },
};
