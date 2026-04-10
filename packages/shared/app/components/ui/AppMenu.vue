<script setup lang="ts">
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuContent,
} from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

defineProps<{
  align?: "start" | "center" | "end";
}>();

const { theme } = useTheme();

const contentClasses = computed(() => {
  const recipe = theme.value.recipes.menu;
  return recipe ? resolveRecipe(recipe, { variant: "content" }) : [];
});

const itemClasses = computed(() => {
  const recipe = theme.value.recipes.menu;
  return recipe ? resolveRecipe(recipe, { variant: "item" }) : [];
});

const separatorClasses = computed(() => {
  const recipe = theme.value.recipes.menu;
  return recipe ? resolveRecipe(recipe, { variant: "separator" }) : [];
});

const labelClasses = computed(() => {
  const recipe = theme.value.recipes.menu;
  return recipe ? resolveRecipe(recipe, { variant: "label" }) : [];
});
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent :class="contentClasses" :align :side-offset="4">
        <slot
          :item-class="itemClasses"
          :separator-class="separatorClasses"
          :label-class="labelClasses"
        />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
