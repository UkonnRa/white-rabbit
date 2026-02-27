<script setup lang="ts">
import type { Journal } from "../models";

defineProps<{
  journal: Journal;
}>();
</script>

<template>
  <div class="journal-card">
    <h3>{{ journal.name }}</h3>
    <p v-if="journal.description">{{ journal.description }}</p>
    <div v-if="journal.tags.length" class="tags">
      <span v-for="tag in journal.tags" :key="tag" class="tag">{{ tag }}</span>
    </div>
  </div>
</template>
