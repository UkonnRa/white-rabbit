<script setup lang="ts">
import {
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectPortal,
  SelectContent,
  SelectViewport,
  SelectItem,
  SelectItemText,
  SelectItemIndicator,
} from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";
import AppIcon from "./AppIcon.vue";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    placeholder?: string;
    options: Array<{ label: string; value: string; disabled?: boolean }>;
  }>(),
  {
    size: "md",
    placeholder: "Select…",
  },
);

const model = defineModel<string>();
const { theme } = useTheme();

const triggerClasses = computed(() => {
  const recipe = theme.value.recipes.select;
  return recipe
    ? resolveRecipe(recipe, {
        variant: "trigger",
        size: props.size,
        disabled: props.disabled,
      })
    : [];
});

const contentClasses = computed(() => {
  const recipe = theme.value.recipes.select;
  return recipe ? resolveRecipe(recipe, { variant: "content" }) : [];
});

const itemClasses = computed(() => {
  const recipe = theme.value.recipes.select;
  return recipe ? resolveRecipe(recipe, { variant: "item" }) : [];
});
</script>

<template>
  <SelectRoot v-model="model" :disabled>
    <SelectTrigger :class="triggerClasses" v-bind="$attrs">
      <SelectValue :placeholder />
    </SelectTrigger>
    <SelectPortal>
      <SelectContent :class="contentClasses" position="popper" :side-offset="4">
        <SelectViewport class="p-1">
          <SelectItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
            :disabled="option.disabled"
            :class="itemClasses"
            class="pl-8 pr-2 py-1.5"
          >
            <SelectItemIndicator
              class="absolute left-2 inline-flex items-center"
            >
              <AppIcon icon="lucide:check" size="sm" />
            </SelectItemIndicator>
            <SelectItemText>{{ option.label }}</SelectItemText>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
