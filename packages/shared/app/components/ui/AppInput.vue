<script setup lang="ts">
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "default" | "error";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
  }>(),
  {
    variant: "default",
    size: "md",
  },
);

const model = defineModel<string>();
const classes = useRecipe("input", props);
</script>

<template>
  <Primitive
    v-model="model"
    as="input"
    :class="classes"
    :disabled
    v-bind="$attrs"
  />
</template>
