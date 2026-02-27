<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import type { Journal } from "../models";
import type { JournalClient } from "../clients";
import JournalCard from "../components/JournalCard.vue";

const journalClient = inject<JournalClient>("journalClient")!;
const journals = ref<Journal[]>([]);
const loading = ref(true);

onMounted(async () => {
  try {
    journals.value = await journalClient.list();
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="journal-list-page">
    <h1>Journals</h1>
    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="journals.length === 0" class="empty">
      No journals yet.
    </div>
    <div v-else class="journal-grid">
      <JournalCard
        v-for="journal in journals"
        :key="journal.id"
        :journal="journal"
      />
    </div>
  </div>
</template>
