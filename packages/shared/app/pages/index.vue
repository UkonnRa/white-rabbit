<script setup lang="ts">
import { ref, computed } from "vue";
import type { JournalFilter, JournalFormData, Journal } from "../models";
import AppButton from "../components/ui/AppButton.vue";

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
  <div class="flex flex-col gap-4">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-bold">Journals</h1>
      <AppButton
        variant="solid"
        :disabled="addingNew"
        @click="addingNew = true"
      >
        + New Journal
      </AppButton>
    </div>

    <div
      v-if="displayError"
      class="rounded-md border border-error bg-error/10 text-error px-4 py-3 text-sm"
    >
      {{ displayError }}
    </div>

    <div
      v-if="status === 'pending'"
      class="text-center text-on-surface-variant"
    >
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
