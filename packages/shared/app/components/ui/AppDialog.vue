<script setup lang="ts">
import {
  DialogRoot,
  DialogTrigger,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from "reka-ui";
import { computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
  }>(),
  {
    size: "md",
  },
);

const open = defineModel<boolean>("open", { default: false });

const { theme } = useTheme();

const overlayClasses = computed(() => {
  const recipe = theme.value.recipes.dialog;
  return recipe ? resolveRecipe(recipe, { variant: "overlay" }) : [];
});

const contentClasses = computed(() => {
  const recipe = theme.value.recipes.dialog;
  return recipe
    ? resolveRecipe(recipe, { variant: "content", size: props.size })
    : [];
});

const titleClasses = computed(() => {
  const recipe = theme.value.recipes.dialog;
  return recipe ? resolveRecipe(recipe, { variant: "title" }) : [];
});

const descriptionClasses = computed(() => {
  const recipe = theme.value.recipes.dialog;
  return recipe ? resolveRecipe(recipe, { variant: "description" }) : [];
});
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogTrigger as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay :class="overlayClasses" />
      <DialogContent :class="contentClasses">
        <DialogTitle v-if="$slots.title" :class="titleClasses">
          <slot name="title" />
        </DialogTitle>
        <DialogDescription
          v-if="$slots.description"
          :class="descriptionClasses"
        >
          <slot name="description" />
        </DialogDescription>
        <slot />
        <DialogClose v-if="$slots.close" as-child>
          <slot name="close" />
        </DialogClose>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
