<script setup lang="ts">
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "default" | "error";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    rows?: number;
  }>(),
  {
    variant: "default",
    size: "md",
    rows: 3,
  },
);

const model = defineModel<string>();
const classes = useRecipe("textarea", props);
</script>

<template>
  <textarea v-model="model" :class="classes" :disabled :rows v-bind="$attrs" />
</template>
