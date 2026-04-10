import { computed, inject } from "vue";
import type { ComponentRecipe, ThemeRecipes } from "../themes/types";
import { THEME_KEY } from "../themes/types";

/**
 * Pure function: resolve a recipe into a flat class list.
 *
 * Exported separately so it can be tested without Vue context.
 */
export function resolveRecipe(
  recipe: ComponentRecipe,
  props: { variant?: string; size?: string; disabled?: boolean },
): string[] {
  const classes: string[] = [...recipe.base];

  // Variant
  if (props.variant && recipe.variants[props.variant]) {
    classes.push(...(recipe.variants[props.variant] || []));
  }

  // Size
  if (props.size && recipe.sizes[props.size]) {
    classes.push(...(recipe.sizes[props.size] || []));
  }

  // Interaction states (hover/focus/pressed are always applied as
  // conditional Tailwind classes like `hover:...`, `focus-visible:...`).
  if (!props.disabled) {
    if (recipe.interactions.hover) classes.push(...recipe.interactions.hover);
    if (recipe.interactions.focus) classes.push(...recipe.interactions.focus);
    if (recipe.interactions.pressed)
      classes.push(...recipe.interactions.pressed);
    if (recipe.interactions.dragged)
      classes.push(...recipe.interactions.dragged);
  }

  // Disabled
  if (props.disabled && recipe.interactions.disabled) {
    classes.push(...recipe.interactions.disabled);
  }

  // Compound variants
  if (recipe.compound) {
    for (const rule of recipe.compound) {
      const variantMatch = !rule.variant || rule.variant === props.variant;
      const sizeMatch = !rule.size || rule.size === props.size;
      if (variantMatch && sizeMatch) {
        classes.push(...rule.classes);
      }
    }
  }

  return classes;
}

/**
 * Composable: inject the current theme and resolve a component recipe.
 *
 * Returns a computed class list that reacts to theme and prop changes.
 */
export function useRecipe(
  component: keyof ThemeRecipes,
  props: { variant?: string; size?: string; disabled?: boolean },
) {
  const theme = inject(THEME_KEY);
  if (!theme) {
    throw new Error(
      "useRecipe: no theme provided. Did you call createThemeRoot?",
    );
  }

  return computed(() => {
    const recipe = theme.value.recipes[component];
    if (!recipe) return [];
    return resolveRecipe(recipe, props);
  });
}
