<script setup lang="ts">
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "solid" | "outlined" | "tonal";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    closable?: boolean;
  }>(),
  {
    variant: "tonal",
    size: "md",
  },
);

defineEmits<{
  close: [];
}>();

const classes = useRecipe("chip", props);
</script>

<template>
  <Primitive as="span" :class="classes" v-bind="$attrs">
    <slot />
    <button
      v-if="closable"
      type="button"
      class="ml-1 inline-flex items-center justify-center rounded-full size-4 hover:bg-black/10"
      aria-label="Remove"
      @click="$emit('close')"
    >
      &#x2715;
    </button>
  </Primitive>
</template>
