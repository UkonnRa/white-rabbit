<script setup lang="ts">
import { ref, computed } from "vue";
import type { Journal, JournalFormData } from "../models";
import AppButton from "../components/ui/AppButton.vue";
import AppIcon from "../components/ui/AppIcon.vue";
import AppInput from "../components/ui/AppInput.vue";
import AppDialog from "../components/ui/AppDialog.vue";
import JournalCard from "../components/JournalCard.vue";
import JournalForm from "../components/JournalForm.vue";

// ── Data ─────���──────────────────────────────────────────────────────────────

const { data: journals, status, error: queryError, refresh } = useJournals();

const client = useJournalClient();
const mutationError = ref<string | null>(null);
const displayError = computed(
  () => mutationError.value ?? queryError.value?.message ?? null,
);

// ── Search filter ───────��───────────────────────────────────────────────────

const searchQuery = ref("");
const showSearch = computed(() => (journals.value?.length ?? 0) >= 5);

const filteredJournals = computed(() => {
  const list = journals.value ?? [];
  const q = searchQuery.value.toLowerCase().trim();
  if (!q) return list;
  return list.filter(
    (j) =>
      j.name.toLowerCase().includes(q) ||
      j.tags.some((t) => t.toLowerCase().includes(q)),
  );
});

// ── Create dialog ──────────��────────────────────────────────────────────────

const showCreateDialog = ref(false);

async function handleCreate(data: JournalFormData) {
  mutationError.value = null;
  try {
    await client.create(data);
    showCreateDialog.value = false;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

// ── Edit dialog ─────────────────────────────────────────────────────────────

const showEditDialog = ref(false);
const editingJournal = ref<Journal | null>(null);

function openEdit(journal: Journal) {
  editingJournal.value = journal;
  showEditDialog.value = true;
}

async function handleUpdate(data: JournalFormData) {
  if (!editingJournal.value) return;
  mutationError.value = null;
  try {
    await client.update(editingJournal.value.id, data);
    showEditDialog.value = false;
    editingJournal.value = null;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}

// ── Delete confirmation ──────���──────────────────���───────────────────────────

const showDeleteDialog = ref(false);
const deletingJournal = ref<Journal | null>(null);
const deleteConfirmName = ref("");

function openDelete(journal: Journal) {
  deletingJournal.value = journal;
  deleteConfirmName.value = "";
  showDeleteDialog.value = true;
}

const deleteEnabled = computed(
  () => deleteConfirmName.value === deletingJournal.value?.name,
);

async function confirmDelete() {
  if (!deletingJournal.value) return;
  mutationError.value = null;
  try {
    await client.delete(deletingJournal.value.id);
    showDeleteDialog.value = false;
    deletingJournal.value = null;
    await refresh();
  } catch (e) {
    mutationError.value = String(e);
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Page header -->
    <div class="flex items-center justify-between gap-4">
      <h1 class="text-xl font-bold text-on-background">Journals</h1>
      <AppButton variant="solid" @click="showCreateDialog = true">
        <AppIcon icon="lucide:plus" size="sm" class="mr-1" />
        New Journal
      </AppButton>
    </div>

    <!-- Search (visible when >= 5 journals) -->
    <AppInput
      v-if="showSearch"
      v-model="searchQuery"
      placeholder="Search journals by name or tag…"
    />

    <!-- Error banner -->
    <div
      v-if="displayError"
      class="rounded-lg border border-error bg-error/10 text-error px-4 py-3 text-sm"
    >
      {{ displayError }}
    </div>

    <!-- Loading -->
    <div
      v-if="status === 'pending'"
      class="text-center py-16 text-on-surface-variant"
    >
      Loading…
    </div>

    <!-- Empty state -->
    <div
      v-else-if="!journals?.length"
      class="flex flex-col items-center justify-center py-20 text-center"
    >
      <AppIcon
        icon="lucide:book-open"
        size="lg"
        class="text-on-surface-variant/40 mb-4 !size-12"
      />
      <p class="text-on-surface-variant text-sm mb-4">
        No journals yet. Create your first ledger to start tracking finances.
      </p>
      <AppButton variant="solid" @click="showCreateDialog = true">
        <AppIcon icon="lucide:plus" size="sm" class="mr-1" />
        Create Journal
      </AppButton>
    </div>

    <!-- No search results -->
    <div
      v-else-if="searchQuery && !filteredJournals.length"
      class="text-center py-12 text-on-surface-variant text-sm"
    >
      No journals match your search.
    </div>

    <!-- Journal card grid -->
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <JournalCard
        v-for="journal in filteredJournals"
        :key="journal.id"
        :journal="journal"
        @click="navigateTo(`/journals/${journal.id}`)"
        @edit="openEdit(journal)"
        @delete="openDelete(journal)"
      />
    </div>

    <!-- Create dialog -->
    <AppDialog v-model:open="showCreateDialog" size="md">
      <template #title>New Journal</template>
      <JournalForm @submit="handleCreate" @cancel="showCreateDialog = false" />
    </AppDialog>

    <!-- Edit dialog -->
    <AppDialog v-model:open="showEditDialog" size="md">
      <template #title>Edit Journal</template>
      <JournalForm
        v-if="editingJournal"
        :initial="{
          name: editingJournal.name,
          description: editingJournal.description,
          tags: editingJournal.tags,
        }"
        @submit="handleUpdate"
        @cancel="showEditDialog = false"
      />
    </AppDialog>

    <!-- Delete confirmation dialog -->
    <AppDialog v-model:open="showDeleteDialog" size="sm">
      <template #title>Delete Journal</template>
      <template #description>
        Are you sure you want to delete
        <strong>{{ deletingJournal?.name }}</strong
        >? All accounts and records within this journal will be permanently
        removed. This action cannot be undone.
      </template>
      <div class="flex flex-col gap-4 pt-4">
        <div>
          <label for="delete-confirm" class="block text-sm mb-1">
            Please input the exact name '<strong>{{
              deletingJournal?.name
            }}</strong
            >' to confirm.
          </label>
          <AppInput
            id="delete-confirm"
            v-model="deleteConfirmName"
            :placeholder="deletingJournal?.name ?? ''"
          />
        </div>
        <div class="flex justify-end gap-2">
          <AppButton variant="outlined" @click="showDeleteDialog = false">
            Cancel
          </AppButton>
          <AppButton
            variant="solid"
            class="bg-error text-on-error"
            :disabled="!deleteEnabled"
            @click="confirmDelete"
          >
            Delete
          </AppButton>
        </div>
      </div>
    </AppDialog>
  </div>
</template>
