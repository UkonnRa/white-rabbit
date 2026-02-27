<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import type { Journal, JournalFilter as JournalFilterType } from "../models";
import type { JournalClient } from "../clients";
import JournalCard from "../components/JournalCard.vue";
import JournalForm, { type JournalFormData } from "../components/JournalForm.vue";
import JournalFilterBar from "../components/JournalFilter.vue";

const journalClient = inject<JournalClient>("journalClient")!;

const journals = ref<Journal[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const currentFilter = ref<JournalFilterType>({});

const showCreateForm = ref(false);
const editingJournal = ref<Journal | null>(null);

async function loadJournals(filter?: JournalFilterType) {
  loading.value = true;
  error.value = null;
  try {
    if (filter !== undefined) currentFilter.value = filter;
    journals.value = await journalClient.list(currentFilter.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function handleCreate(data: JournalFormData) {
  error.value = null;
  try {
    await journalClient.create(data);
    showCreateForm.value = false;
    await loadJournals();
  } catch (e) {
    error.value = String(e);
  }
}

async function handleUpdate(data: JournalFormData) {
  if (!editingJournal.value) return;
  error.value = null;
  try {
    await journalClient.update(editingJournal.value.id, data);
    editingJournal.value = null;
    await loadJournals();
  } catch (e) {
    error.value = String(e);
  }
}

async function handleDelete(journal: Journal) {
  error.value = null;
  try {
    await journalClient.delete(journal.id);
    await loadJournals();
  } catch (e) {
    error.value = String(e);
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

onMounted(() => loadJournals());
</script>

<template>
  <div class="journal-list-page">
    <div class="page-header">
      <h1>Journals</h1>
      <button @click="startCreate">+ New Journal</button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <JournalFilterBar @search="loadJournals" />

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

    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="journals.length === 0" class="empty">
      No journals found.
    </div>
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
