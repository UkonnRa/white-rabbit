<script setup lang="ts">
import type { Journal } from "../models";
import AppCard from "./ui/AppCard.vue";
import AppButton from "./ui/AppButton.vue";
import AppChip from "./ui/AppChip.vue";

defineProps<{
  journal: Journal;
}>();

defineEmits<{
  edit: [journal: Journal];
  delete: [journal: Journal];
}>();
</script>

<template>
  <AppCard variant="elevated" class="flex flex-col">
    <div class="flex items-start justify-between gap-2">
      <h3 class="text-sm font-medium">
        {{ journal.name }}
      </h3>
      <div class="flex gap-1 shrink-0">
        <AppButton variant="outlined" size="sm" @click="$emit('edit', journal)">
          Edit
        </AppButton>
        <AppButton
          variant="solid"
          size="sm"
          class="bg-error text-on-error"
          @click="$emit('delete', journal)"
        >
          Delete
        </AppButton>
      </div>
    </div>
    <p v-if="journal.description" class="mt-1 text-sm text-on-surface-variant">
      {{ journal.description }}
    </p>
    <div v-if="journal.tags.length" class="mt-auto pt-3 flex flex-wrap gap-1">
      <AppChip v-for="tag in journal.tags" :key="tag" variant="tonal" size="sm">
        {{ tag }}
      </AppChip>
    </div>
  </AppCard>
</template>
