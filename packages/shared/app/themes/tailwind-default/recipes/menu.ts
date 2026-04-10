import type { ComponentRecipe } from "../../types";

export const menuRecipe: ComponentRecipe = {
  base: [],
  variants: {
    content: [
      "min-w-[8rem]",
      "overflow-hidden",
      "rounded-md",
      "border",
      "border-outline-variant",
      "bg-surface",
      "text-on-surface",
      "shadow-lg",
      "p-1",
    ],
    item: [
      "relative",
      "flex",
      "items-center",
      "gap-2",
      "rounded-sm",
      "px-2",
      "py-1.5",
      "text-sm",
      "cursor-pointer",
      "select-none",
      "outline-none",
      "data-[highlighted]:bg-surface-variant",
      "data-[highlighted]:text-on-surface-variant",
    ],
    separator: ["-mx-1", "my-1", "h-px", "bg-outline-variant"],
    label: [
      "px-2",
      "py-1.5",
      "text-xs",
      "font-semibold",
      "text-on-surface-variant",
    ],
  },
  sizes: {},
  interactions: {
    disabled: ["opacity-50", "pointer-events-none"],
  },
};
