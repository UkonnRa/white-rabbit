<script setup lang="ts">
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg" | "xl";
  }>(),
  {
    size: "md",
  },
);

const classes = useRecipe("icon", props);
</script>

<template>
  <Primitive as="span" :class="classes" aria-hidden="true" v-bind="$attrs">
    <slot />
  </Primitive>
</template>
