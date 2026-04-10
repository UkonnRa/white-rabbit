<script setup lang="ts">
import {
  ComboboxRoot,
  ComboboxInput,
  ComboboxPortal,
  ComboboxContent,
  ComboboxViewport,
  ComboboxItem,
  ComboboxItemIndicator,
  ComboboxEmpty,
} from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    placeholder?: string;
    options: Array<{ label: string; value: string; disabled?: boolean }>;
    multiple?: boolean;
  }>(),
  {
    size: "md",
    placeholder: "Search…",
  },
);

const model = defineModel<string | string[]>();
const { theme } = useTheme();

const inputClasses = computed(() => {
  const recipe = theme.value.recipes.combobox;
  return recipe
    ? resolveRecipe(recipe, {
        variant: "input",
        size: props.size,
        disabled: props.disabled,
      })
    : [];
});

const contentClasses = computed(() => {
  const recipe = theme.value.recipes.combobox;
  return recipe ? resolveRecipe(recipe, { variant: "content" }) : [];
});

const itemClasses = computed(() => {
  const recipe = theme.value.recipes.combobox;
  return recipe ? resolveRecipe(recipe, { variant: "item" }) : [];
});

const emptyClasses = computed(() => {
  const recipe = theme.value.recipes.combobox;
  return recipe ? resolveRecipe(recipe, { variant: "empty" }) : [];
});
</script>

<template>
  <ComboboxRoot v-model="model" :multiple :disabled>
    <ComboboxInput :class="inputClasses" :placeholder v-bind="$attrs" />
    <ComboboxPortal>
      <ComboboxContent
        :class="contentClasses"
        position="popper"
        :side-offset="4"
      >
        <ComboboxViewport class="p-1">
          <ComboboxItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
            :disabled="option.disabled"
            :class="itemClasses"
          >
            <ComboboxItemIndicator
              class="absolute left-2 inline-flex items-center"
            >
              &#x2713;
            </ComboboxItemIndicator>
            <span>{{ option.label }}</span>
          </ComboboxItem>
          <ComboboxEmpty :class="emptyClasses">
            No results found.
          </ComboboxEmpty>
        </ComboboxViewport>
      </ComboboxContent>
    </ComboboxPortal>
  </ComboboxRoot>
</template>
