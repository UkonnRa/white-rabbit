<script setup lang="ts">
import { Primitive } from "reka-ui";
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
  <Primitive
    v-model="model"
    as="textarea"
    :class="classes"
    :disabled
    :rows
    v-bind="$attrs"
  />
</template>
