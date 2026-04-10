<script setup lang="ts">
import type { Component } from "vue";
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "solid" | "outlined" | "ghost" | "text";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    as?: string | Component;
  }>(),
  {
    variant: "solid",
    size: "md",
    as: "button",
  },
);

const classes = useRecipe("button", props);
</script>

<template>
  <Primitive :as="as" :class="classes" :disabled v-bind="$attrs">
    <slot />
  </Primitive>
</template>
