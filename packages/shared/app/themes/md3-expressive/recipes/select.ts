import type { ComponentRecipe } from "../../types";

export const selectRecipe: ComponentRecipe = {
  base: [],
  variants: {
    trigger: [
      "md3-state-layer",
      "inline-flex",
      "items-center",
      "justify-between",
      "gap-2",
      "rounded-xl",
      "border",
      "border-outline-variant",
      "bg-surface-container",
      "text-on-surface",
      "transition-colors",
      "duration-normal",
      "ease-standard",
      "cursor-pointer",
    ],
    content: [
      "overflow-hidden",
      "rounded-xl",
      "bg-surface-container",
      "text-on-surface",
      "shadow-[var(--wr-elevation-2)]",
    ],
    item: [
      "md3-state-layer",
      "relative",
      "flex",
      "items-center",
      "rounded-lg",
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
    sm: ["h-10", "px-3", "text-sm"],
    md: ["h-14", "px-4", "text-base"],
    lg: ["h-16", "px-5", "text-lg"],
  },
  interactions: {
    hover: ["hover:before:opacity-[0.08]"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-0",
      "focus-visible:outline-primary",
    ],
    disabled: ["opacity-[0.38]", "pointer-events-none"],
  },
};
