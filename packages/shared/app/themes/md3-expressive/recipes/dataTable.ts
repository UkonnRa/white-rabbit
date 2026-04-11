import type { ComponentRecipe } from "../../types";

export const dataTableRecipe: ComponentRecipe = {
  base: [],
  variants: {
    table: ["w-full", "caption-bottom", "text-sm", "border-collapse"],
    header: ["border-b", "border-outline-variant"],
    headerCell: [
      "h-14",
      "px-4",
      "text-left",
      "align-middle",
      "font-medium",
      "text-on-surface-variant",
      "tracking-wide",
    ],
    body: [],
    row: [
      "border-b",
      "border-outline-variant",
      "transition-colors",
      "duration-fast",
    ],
    cell: ["px-4", "py-3", "align-middle", "text-on-surface"],
    empty: ["px-4", "py-8", "text-center", "text-on-surface-variant"],
  },
  sizes: {
    sm: [],
    md: [],
    lg: [],
  },
  interactions: {
    hover: ["hover:bg-on-surface/[0.04]"],
  },
};
