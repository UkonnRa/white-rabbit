<script setup lang="ts">
import type { Journal } from "../models";

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
  <UCard
    class="cursor-pointer transition-colors hover:border-(--ui-primary)"
    role="link"
    tabindex="0"
    @click="emit('click')"
    @keydown.enter="emit('click')"
  >
    <div class="flex items-start justify-between gap-2">
      <h3 class="text-base font-semibold truncate">
        {{ journal.name }}
      </h3>
      <div class="flex items-center gap-1 shrink-0">
        <UButton
          variant="ghost"
          size="sm"
          icon="lucide:pencil"
          aria-label="Edit journal"
          @click.stop="emit('edit')"
        />
        <UButton
          variant="ghost"
          size="sm"
          icon="lucide:trash-2"
          color="error"
          aria-label="Delete journal"
          @click.stop="emit('delete')"
        />
      </div>
    </div>

    <p
      class="text-sm leading-relaxed line-clamp-2"
      :class="
        journal.description
          ? 'text-(--ui-text-dimmed)'
          : 'text-(--ui-text-dimmed)/50 italic'
      "
    >
      {{ journal.description || "No description" }}
    </p>

    <div v-if="journal.tags.length" class="flex flex-wrap gap-1">
      <UBadge v-for="tag in journal.tags" :key="tag" variant="soft" size="sm">
        {{ tag }}
      </UBadge>
    </div>

    <div class="mt-auto pt-1 text-xs text-(--ui-text-dimmed)/70">
      <span v-if="journal.last_modified_at">
        Updated {{ formatRelativeDate(journal.last_modified_at) }}
      </span>
      <span v-else-if="journal.created_at">
        Created {{ formatRelativeDate(journal.created_at) }}
      </span>
    </div>
  </UCard>
</template>
