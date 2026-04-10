import type { ComponentRecipe } from "../../types";

export const buttonRecipe: ComponentRecipe = {
  base: [
    "inline-flex",
    "items-center",
    "justify-center",
    "gap-2",
    "rounded-md",
    "font-medium",
    "whitespace-nowrap",
    "transition-all",
    "duration-normal",
    "ease-standard",
    "cursor-pointer",
    "select-none",
  ],
  variants: {
    solid: ["bg-primary", "text-on-primary", "shadow-sm"],
    outlined: ["border", "border-outline", "text-primary", "bg-transparent"],
    ghost: ["text-on-surface", "bg-transparent"],
    text: ["text-primary", "bg-transparent"],
  },
  sizes: {
    sm: ["h-8", "px-3", "text-sm"],
    md: ["h-10", "px-4", "text-sm"],
    lg: ["h-12", "px-6", "text-base"],
  },
  interactions: {
    hover: ["hover:brightness-110"],
    focus: [
      "focus-visible:outline-2",
      "focus-visible:outline-offset-2",
      "focus-visible:outline-primary",
    ],
    pressed: ["active:scale-[0.98]"],
    disabled: ["opacity-50", "pointer-events-none", "cursor-not-allowed"],
  },
  compound: [
    { variant: "ghost", classes: ["hover:bg-surface-variant"] },
    { variant: "text", classes: ["hover:bg-surface-variant"] },
  ],
};
