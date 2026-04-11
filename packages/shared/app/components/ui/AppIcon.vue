<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    icon: string;
    size?: "sm" | "md" | "lg" | "xl";
  }>(),
  {
    size: "md",
  },
);

const classes = useRecipe("icon", props);

const sizeMap: Record<string, string> = {
  sm: "16",
  md: "20",
  lg: "24",
  xl: "32",
};
</script>

<template>
  <Icon
    :icon="icon"
    :width="sizeMap[size]"
    :height="sizeMap[size]"
    :class="classes"
    aria-hidden="true"
    v-bind="$attrs"
  />
</template>
