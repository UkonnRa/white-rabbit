<script setup lang="ts">
import type { Component } from "vue";
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "elevated" | "outlined" | "filled";
    size?: "sm" | "md" | "lg";
    as?: string | Component;
  }>(),
  {
    variant: "elevated",
    size: "md",
    as: "div",
  },
);

const classes = useRecipe("card", props);
</script>

<template>
  <Primitive :as="as" :class="classes" v-bind="$attrs">
    <slot />
  </Primitive>
</template>
