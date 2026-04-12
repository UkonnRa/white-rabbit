<script setup lang="ts">
import type { Journal } from "../models";
import AppCard from "./ui/AppCard.vue";
import AppButton from "./ui/AppButton.vue";
import AppIcon from "./ui/AppIcon.vue";
import AppChip from "./ui/AppChip.vue";

defineProps<{
  journal: Journal;
}>();

const emit = defineEmits<{
  edit: [];
  delete: [];
  click: [];
}>();

function formatRelativeDate(dateStr: string | null): string {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 30) return `${diffDays} days ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
  return date.toLocaleDateString();
}
</script>

<template>
  <AppCard
    variant="outlined"
    class="flex flex-col gap-3 p-4 cursor-pointer transition-colors hover:border-primary"
    role="link"
    tabindex="0"
    @click="emit('click')"
    @keydown.enter="emit('click')"
  >
    <!-- Header: name + actions -->
    <div class="flex items-start justify-between gap-2">
      <h3 class="text-base font-semibold text-on-surface truncate">
        {{ journal.name }}
      </h3>
      <div class="flex items-center gap-1 shrink-0">
        <AppButton
          variant="ghost"
          size="sm"
          aria-label="Edit journal"
          @click.stop="emit('edit')"
        >
          <AppIcon icon="lucide:pencil" size="sm" />
        </AppButton>
        <AppButton
          variant="ghost"
          size="sm"
          class="text-error"
          aria-label="Delete journal"
          @click.stop="emit('delete')"
        >
          <AppIcon icon="lucide:trash-2" size="sm" />
        </AppButton>
      </div>
    </div>

    <!-- Description -->
    <p
      class="text-sm leading-relaxed line-clamp-2"
      :class="
        journal.description
          ? 'text-on-surface-variant'
          : 'text-on-surface-variant/50 italic'
      "
    >
      {{ journal.description || "No description" }}
    </p>

    <!-- Tags -->
    <div v-if="journal.tags.length" class="flex flex-wrap gap-1">
      <AppChip v-for="tag in journal.tags" :key="tag" variant="tonal" size="sm">
        {{ tag }}
      </AppChip>
    </div>

    <!-- Timestamps -->
    <div class="mt-auto pt-1 text-xs text-on-surface-variant/70">
      <span v-if="journal.last_modified_at">
        Updated {{ formatRelativeDate(journal.last_modified_at) }}
      </span>
      <span v-else-if="journal.created_at">
        Created {{ formatRelativeDate(journal.created_at) }}
      </span>
    </div>
  </AppCard>
</template>
