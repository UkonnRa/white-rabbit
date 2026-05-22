<script setup lang="ts">
import { ref, computed } from "vue";
import type { Journal, JournalFormData } from "../models";
import JournalCard from "../components/JournalCard.vue";
import JournalForm from "../components/JournalForm.vue";

const router = useRouter();
const _navigateTo = navigateTo;

const { data: journals, status, error: queryError, refresh } = useJournals();

const client = useJournalClient();
const mutationError = ref<string | null>(null);
const displayError = computed(
  () => mutationError.value ?? queryError.value?.message ?? null,
);

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
    <div class="flex items-center justify-between gap-4">
      <h1 class="text-xl font-bold">Journals</h1>
      <UButton @click="showCreateDialog = true">
        <UIcon name="lucide:plus" class="mr-1" />
        New Journal
      </UButton>
    </div>

    <UInput
      v-if="showSearch"
      v-model="searchQuery"
      placeholder="Search journals by name or tag..."
    />

    <div
      v-if="displayError"
      class="rounded-lg border border-(--ui-error) bg-(--ui-error)/10 text-(--ui-error) px-4 py-3 text-sm"
    >
      {{ displayError }}
    </div>

    <div
      v-if="status === 'pending'"
      class="text-center py-16 text-(--ui-text-dimmed)"
    >
      Loading...
    </div>

    <div
      v-else-if="!journals?.length"
      class="flex flex-col items-center justify-center py-20 text-center"
    >
      <UIcon
        name="lucide:book-open"
        class="text-(--ui-text-dimmed)/40 mb-4 size-12"
      />
      <p class="text-(--ui-text-dimmed) text-sm mb-4">
        No journals yet. Create your first ledger to start tracking finances.
      </p>
      <UButton @click="showCreateDialog = true">
        <UIcon name="lucide:plus" class="mr-1" />
        Create Journal
      </UButton>
    </div>

    <div
      v-else-if="searchQuery && !filteredJournals.length"
      class="text-center py-12 text-(--ui-text-dimmed) text-sm"
    >
      No journals match your search.
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <JournalCard
        v-for="journal in filteredJournals"
        :key="journal.id"
        :journal="journal"
        @click="router.push(`/journals/${journal.id}`)"
        @edit="openEdit(journal)"
        @delete="openDelete(journal)"
      />
    </div>

    <UModal v-model:open="showCreateDialog" title="New Journal">
      <template #body>
        <JournalForm @submit="handleCreate" @cancel="showCreateDialog = false" />
      </template>
    </UModal>

    <UModal v-model:open="showEditDialog" title="Edit Journal">
      <template #body>
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
      </template>
    </UModal>

    <UModal v-model:open="showDeleteDialog" title="Delete Journal">
      <template #body>
        <p class="text-sm mb-4">
          Are you sure you want to delete
          <strong>{{ deletingJournal?.name }}</strong
          >? All accounts and records within this journal will be permanently
          removed. This action cannot be undone.
        </p>
        <div class="flex flex-col gap-4">
          <div>
            <label for="delete-confirm" class="block text-sm mb-1">
              Type <strong>{{ deletingJournal?.name }}</strong> to confirm:
            </label>
            <UInput
              id="delete-confirm"
              v-model="deleteConfirmName"
              :placeholder="deletingJournal?.name ?? ''"
            />
          </div>
          <div class="flex justify-end gap-2">
            <UButton variant="outline" @click="showDeleteDialog = false">
              Cancel
            </UButton>
            <UButton
              color="error"
              :disabled="!deleteEnabled"
              @click="confirmDelete"
            >
              Delete
            </UButton>
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>
