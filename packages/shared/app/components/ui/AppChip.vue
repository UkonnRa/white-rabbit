<script setup lang="ts">
import { ref } from "vue";
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";
import { useRipple } from "../../composables/useRipple";
import AppIcon from "./AppIcon.vue";

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
const elRef = ref<HTMLElement | null>(null);
useRipple(elRef);
</script>

<template>
  <Primitive ref="elRef" as="span" :class="classes" v-bind="$attrs">
    <slot />
    <button
      v-if="closable"
      type="button"
      class="ml-1 inline-flex items-center justify-center rounded-full size-4 hover:bg-black/10"
      aria-label="Remove"
      @click="$emit('close')"
    >
      <AppIcon icon="lucide:x" size="sm" />
    </button>
  </Primitive>
</template>
