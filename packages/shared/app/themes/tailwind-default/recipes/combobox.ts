import type { ComponentRecipe } from "../../types";

export const comboboxRecipe: ComponentRecipe = {
  base: [],
  variants: {
    input: [
      "w-full",
      "rounded-md",
      "border",
      "border-outline-variant",
      "bg-surface",
      "text-on-surface",
      "placeholder:text-on-surface-variant",
      "transition-colors",
      "duration-normal",
      "ease-standard",
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
      "px-2",
      "py-1.5",
      "text-sm",
      "cursor-pointer",
      "select-none",
      "outline-none",
      "data-[highlighted]:bg-surface-variant",
      "data-[highlighted]:text-on-surface-variant",
    ],
    empty: [
      "px-2",
      "py-6",
      "text-center",
      "text-sm",
      "text-on-surface-variant",
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
      "focus:outline-2",
      "focus:outline-offset-0",
      "focus:outline-primary",
      "focus:border-primary",
    ],
    disabled: ["opacity-50", "pointer-events-none"],
  },
};
