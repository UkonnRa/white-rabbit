import type { ComponentRecipe } from "../../types";

export const dataTableRecipe: ComponentRecipe = {
  base: [],
  variants: {
    table: ["w-full", "caption-bottom", "text-sm", "border-collapse"],
    header: ["border-b", "border-outline-variant"],
    headerCell: [
      "h-10",
      "px-3",
      "text-left",
      "align-middle",
      "font-medium",
      "text-on-surface-variant",
    ],
    body: [],
    row: [
      "border-b",
      "border-outline-variant",
      "transition-colors",
      "duration-fast",
    ],
    cell: ["px-3", "py-2", "align-middle", "text-on-surface"],
    empty: ["px-3", "py-6", "text-center", "text-on-surface-variant"],
  },
  sizes: {
    sm: [],
    md: [],
    lg: [],
  },
  interactions: {
    hover: ["hover:bg-surface-variant/50"],
  },
};
