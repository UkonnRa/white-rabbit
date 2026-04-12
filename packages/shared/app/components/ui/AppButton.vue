<script setup lang="ts">
import type { Component } from "vue";
import { ref } from "vue";
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";
import { useRipple } from "../../composables/useRipple";

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
const elRef = ref<HTMLElement | null>(null);
useRipple(elRef);
</script>

<template>
  <Primitive ref="elRef" :as="as" :class="classes" :disabled v-bind="$attrs">
    <slot />
  </Primitive>
</template>
