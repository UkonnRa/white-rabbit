<script setup lang="ts">
import {
  TooltipRoot,
  TooltipTrigger,
  TooltipPortal,
  TooltipContent,
  TooltipArrow,
} from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

defineProps<{
  side?: "top" | "right" | "bottom" | "left";
  text?: string;
}>();

const { theme } = useTheme();

const contentClasses = computed(() => {
  const recipe = theme.value.recipes.tooltip;
  return recipe ? resolveRecipe(recipe, { variant: "content" }) : [];
});
</script>

<template>
  <TooltipRoot>
    <TooltipTrigger as-child>
      <slot />
    </TooltipTrigger>
    <TooltipPortal>
      <TooltipContent :class="contentClasses" :side :side-offset="4">
        <slot name="content">{{ text }}</slot>
        <TooltipArrow class="fill-on-surface" />
      </TooltipContent>
    </TooltipPortal>
  </TooltipRoot>
</template>
