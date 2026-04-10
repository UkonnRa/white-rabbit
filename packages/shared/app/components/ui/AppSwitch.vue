<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
  }>(),
  {
    size: "md",
  },
);

const model = defineModel<boolean>();
const { theme } = useTheme();

const rootClasses = computed(() => {
  const recipe = theme.value.recipes.switch;
  return recipe
    ? resolveRecipe(recipe, {
        variant: "root",
        size: props.size,
        disabled: props.disabled,
      })
    : [];
});

const thumbClasses = computed(() => {
  const recipe = theme.value.recipes.switch;
  return recipe ? resolveRecipe(recipe, { variant: "thumb" }) : [];
});

const thumbSizeMap: Record<string, string> = {
  sm: "size-4 data-[state=checked]:translate-x-4",
  md: "size-5 data-[state=checked]:translate-x-5",
  lg: "size-5 data-[state=checked]:translate-x-6",
};
</script>

<template>
  <SwitchRoot
    v-model:checked="model"
    :class="rootClasses"
    :disabled
    v-bind="$attrs"
  >
    <SwitchThumb :class="[...thumbClasses, thumbSizeMap[size]]" />
  </SwitchRoot>
</template>
