import type { ComponentRecipe } from "../../types";

export const dialogRecipe: ComponentRecipe = {
  base: [],
  variants: {
    overlay: [
      "fixed",
      "inset-0",
      "z-50",
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
      "z-50",
      "-translate-x-1/2",
      "-translate-y-1/2",
      "w-full",
      "rounded-lg",
      "bg-surface",
      "text-on-surface",
      "shadow-xl",
      "p-6",
      "data-[state=open]:animate-in",
      "data-[state=open]:fade-in-0",
      "data-[state=open]:zoom-in-95",
      "data-[state=closed]:animate-out",
      "data-[state=closed]:fade-out-0",
      "data-[state=closed]:zoom-out-95",
    ],
    title: ["text-lg", "font-semibold", "text-on-surface"],
    description: ["text-sm", "text-on-surface-variant", "mt-2"],
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
