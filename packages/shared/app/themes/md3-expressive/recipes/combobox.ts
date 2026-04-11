import type { ComponentRecipe } from "../../types";

export const comboboxRecipe: ComponentRecipe = {
  base: [],
  variants: {
    input: [
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
      "px-3",
      "py-2",
      "text-sm",
      "cursor-pointer",
      "select-none",
      "outline-none",
      "data-[highlighted]:before:opacity-[0.08]",
    ],
    empty: [
      "px-3",
      "py-6",
      "text-center",
      "text-sm",
      "text-on-surface-variant",
    ],
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
    disabled: ["opacity-[0.38]", "pointer-events-none"],
  },
};
