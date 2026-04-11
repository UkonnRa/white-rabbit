import type { ComponentRecipe } from "../../types";

export const menuRecipe: ComponentRecipe = {
  base: [],
  variants: {
    content: [
      "min-w-[8rem]",
      "overflow-hidden",
      "rounded-xl",
      "bg-surface-container",
      "text-on-surface",
      "shadow-[var(--wr-elevation-2)]",
      "py-2",
    ],
    item: [
      "md3-state-layer",
      "relative",
      "flex",
      "items-center",
      "gap-3",
      "px-3",
      "py-2",
      "text-sm",
      "cursor-pointer",
      "select-none",
      "outline-none",
      "data-[highlighted]:before:opacity-[0.08]",
    ],
    separator: ["-mx-0", "my-2", "h-px", "bg-outline-variant"],
    label: [
      "px-3",
      "py-2",
      "text-xs",
      "font-medium",
      "text-on-surface-variant",
      "tracking-wide",
      "uppercase",
    ],
  },
  sizes: {},
  interactions: {
    disabled: ["opacity-[0.38]", "pointer-events-none"],
  },
};
