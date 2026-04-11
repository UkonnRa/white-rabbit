import type { ComponentRecipe } from "../../types";

export const dialogRecipe: ComponentRecipe = {
  base: [],
  variants: {
    overlay: [
      "fixed",
      "inset-0",
      "bg-black/50",
      "data-[state=open]:animate-in",
      "data-[state=open]:fade-in-0",
      "data-[state=closed]:animate-out",
      "data-[state=closed]:fade-out-0",
    ],
    content: [
      "fixed",
      "top-1/2",
      "left-1/2",
      "-translate-x-1/2",
      "-translate-y-1/2",
      "w-full",
      "rounded-[1.75rem]",
      "bg-surface-container-high",
      "text-on-surface",
      "shadow-[var(--wr-elevation-3)]",
      "p-6",
      "data-[state=open]:animate-in",
      "data-[state=open]:fade-in-0",
      "data-[state=open]:zoom-in-95",
      "data-[state=closed]:animate-out",
      "data-[state=closed]:fade-out-0",
      "data-[state=closed]:zoom-out-95",
    ],
    title: ["text-2xl", "font-normal", "text-on-surface"],
    description: ["text-sm", "text-on-surface-variant", "mt-4"],
  },
  sizes: {
    sm: ["max-w-sm"],
    md: ["max-w-lg"],
    lg: ["max-w-2xl"],
  },
  interactions: {
    focus: ["focus:outline-none"],
  },
};
