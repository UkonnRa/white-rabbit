<script setup lang="ts">
import type { Journal } from "../../models";

defineProps<{
  journal: Journal;
}>();

defineEmits<{
  edit: [journal: Journal];
  delete: [journal: Journal];
}>();
</script>

<template>
  <div class="journal-card">
    <div class="journal-card-header">
      <h3>{{ journal.name }}</h3>
      <div class="journal-card-actions">
        <UiButton @click="$emit('edit', journal)">Edit</UiButton>
        <UiButton @click="$emit('delete', journal)">Delete</UiButton>
      </div>
    </div>
    <p v-if="journal.description" class="journal-description">
      {{ journal.description }}
    </p>
    <div v-if="journal.tags.length" class="tags">
      <span v-for="tag in journal.tags" :key="tag" class="tag">{{ tag }}</span>
    </div>
  </div>
</template>
