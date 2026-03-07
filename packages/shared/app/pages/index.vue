<script setup lang="ts">
import type { Journal, JournalFilter, JournalFormData } from "../../models";

const currentFilter = ref<JournalFilter>({});
const {
  data: journals,
  status,
  error: queryError,
  refresh,
} = useJournals(currentFilter);

const client = useJournalClient();

const showCreateForm = ref(false);
const editingJournal = ref<Journal | null>(null);
const mutationError = ref<string | null>(null);

const displayError = computed(
  () => mutationError.value ?? queryError.value?.message ?? null,
);

function handleFilter(filter: JournalFilter) {
  currentFilter.value = filter;
}

async function handleCreate(data: JournalFormData) {
  mutationError.value = null;
  try {
    await client.create(data);
    showCreateForm.value = false;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

async function handleUpdate(data: JournalFormData) {
  if (!editingJournal.value) return;
  mutationError.value = null;
  try {
    await client.update(editingJournal.value.id, data);
    editingJournal.value = null;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

async function handleDelete(journal: Journal) {
  mutationError.value = null;
  try {
    await client.delete(journal.id);
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

function startEdit(journal: Journal) {
  editingJournal.value = journal;
  showCreateForm.value = false;
}

function startCreate() {
  showCreateForm.value = true;
  editingJournal.value = null;
}

function cancelForm() {
  showCreateForm.value = false;
  editingJournal.value = null;
}
</script>

<template>
  <div class="journal-list-page">
    <div class="page-header">
      <h1>Journals</h1>
      <button @click="startCreate">+ New Journal</button>
    </div>

    <div v-if="displayError" class="error">{{ displayError }}</div>

    <JournalFilterBar @search="handleFilter" />

    <JournalForm
      v-if="showCreateForm"
      @submit="handleCreate"
      @cancel="cancelForm"
    />

    <JournalForm
      v-if="editingJournal"
      :initial="{
        name: editingJournal.name,
        description: editingJournal.description,
        tags: editingJournal.tags,
      }"
      @submit="handleUpdate"
      @cancel="cancelForm"
    />

    <div v-if="status === 'pending'" class="loading">Loading...</div>
    <div v-else-if="!journals?.length" class="empty">No journals found.</div>
    <div v-else class="journal-grid">
      <JournalCard
        v-for="journal in journals"
        :key="journal.id"
        :journal="journal"
        @edit="startEdit"
        @delete="handleDelete"
      />
    </div>
  </div>
</template>
