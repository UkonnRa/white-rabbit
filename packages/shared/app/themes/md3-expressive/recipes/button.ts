import type { ComponentRecipe } from "../../types";

export const buttonRecipe: ComponentRecipe = {
  base: [
    "md3-state-layer",
    "inline-flex",
    "items-center",
    "justify-center",
    "gap-2",
    "rounded-full",
    "font-medium",
    "whitespace-nowrap",
    "transition-all",
    "duration-normal",
    "ease-emphasized",
    "cursor-pointer",
    "select-none",
  ],
  variants: {
    solid: ["bg-primary", "text-on-primary", "shadow-[var(--wr-elevation-1)]"],
    outlined: ["border", "border-outline", "text-primary", "bg-transparent"],
    ghost: ["text-on-surface", "bg-transparent"],
    text: ["text-primary", "bg-transparent"],
  },
  sizes: {
    sm: ["h-8", "px-4", "text-sm"],
    md: ["h-10", "px-6", "text-sm"],
    lg: ["h-14", "px-8", "text-base"],
  },
  interactions: {
    hover: [
      "hover:before:opacity-[0.08]",
      "hover:shadow-[var(--wr-elevation-2)]",
    ],
    focus: ["focus-visible:before:opacity-[0.10]"],
    pressed: ["active:before:opacity-[0.10]"],
    disabled: ["opacity-[0.38]", "pointer-events-none", "shadow-none"],
  },
  compound: [
    // Outlined/ghost/text don't get elevation
    { variant: "outlined", classes: ["hover:shadow-none", "shadow-none"] },
    { variant: "ghost", classes: ["hover:shadow-none", "shadow-none"] },
    { variant: "text", classes: ["hover:shadow-none", "shadow-none"] },
  ],
};
