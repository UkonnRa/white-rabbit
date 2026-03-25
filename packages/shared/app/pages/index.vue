<script setup lang="ts">
import type { JournalFilter, JournalFormData, Journal } from "~/models";

const currentFilter = ref<JournalFilter>({});
const {
  data: journals,
  status,
  error: queryError,
  refresh,
} = useJournals(currentFilter);

const client = useJournalClient();

const addingNew = ref(false);
const mutationError = ref<string | null>(null);

const displayError = computed(
  () => mutationError.value ?? queryError.value?.message ?? null,
);

async function handleCreate(data: JournalFormData) {
  mutationError.value = null;
  try {
    await client.create(data);
    addingNew.value = false;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

async function handleUpdate(id: string, data: JournalFormData) {
  mutationError.value = null;
  try {
    await client.update(id, data);
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
</script>

<template>
  <div class="d-flex flex-column ga-4">
    <div class="d-flex align-center justify-space-between">
      <h1 class="text-h5 font-weight-bold">Journals</h1>
      <v-btn
        color="primary"
        :disabled="addingNew"
        prepend-icon="mdi-plus"
        @click="addingNew = true"
      >
        New Journal
      </v-btn>
    </div>

    <v-alert v-if="displayError" type="error" variant="tonal" closable>
      {{ displayError }}
    </v-alert>

    <div v-if="status === 'pending'" class="text-center text-medium-emphasis">
      Loading...
    </div>
    <JournalTable
      v-else
      :journals="journals ?? []"
      :adding-new="addingNew"
      @create="handleCreate"
      @update="handleUpdate"
      @delete="handleDelete"
      @cancel-new="addingNew = false"
    />
  </div>
</template>
