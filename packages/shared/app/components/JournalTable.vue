<script setup lang="ts">
import { ref, computed, watch } from "vue";
import type { ColumnDef } from "@tanstack/vue-table";
import type { Journal, JournalFormData } from "../models";
import AppDataTable from "./ui/AppDataTable.vue";
import AppInput from "./ui/AppInput.vue";
import AppButton from "./ui/AppButton.vue";
import AppChip from "./ui/AppChip.vue";
import AppIcon from "./ui/AppIcon.vue";
import AppTagInput from "./ui/AppTagInput.vue";

const props = defineProps<{
  journals: Journal[];
  addingNew: boolean;
}>();

const emit = defineEmits<{
  create: [data: JournalFormData];
  update: [id: string, data: JournalFormData];
  delete: [journal: Journal];
  cancelNew: [];
}>();

// ── Column filters ──────────────────────────────────────────────────────────

const nameFilter = ref("");
const descFilter = ref("");
const tagsFilter = ref("");

const filteredJournals = computed(() => {
  return props.journals
    .filter((j) => {
      if (
        nameFilter.value &&
        !j.name.toLowerCase().includes(nameFilter.value.toLowerCase())
      )
        return false;
      if (
        descFilter.value &&
        !(j.description ?? "")
          .toLowerCase()
          .includes(descFilter.value.toLowerCase())
      )
        return false;
      if (
        tagsFilter.value &&
        !j.tags.some((t) =>
          t.toLowerCase().includes(tagsFilter.value.toLowerCase()),
        )
      )
        return false;
      return true;
    })
    .sort((a, b) => (a.name || "").localeCompare(b.name || ""));
});

// ── All known tags (suggestions for tag input) ─────────────────────────────

const allTags = computed(() => {
  const tags = new Set<string>();
  for (const j of props.journals) {
    for (const t of j.tags) tags.add(t);
  }
  return [...tags].sort();
});

// ── Inline editing state ────────────────────────────────────────────────────

const editingId = ref<string | null>(null);
const editForm = ref<JournalFormData>({ name: "", description: "", tags: [] });
const newForm = ref<JournalFormData>({ name: "", description: "", tags: [] });

watch(
  () => props.addingNew,
  (val) => {
    if (val) {
      newForm.value = { name: "", description: "", tags: [] };
      editingId.value = null;
    }
  },
);

function startEdit(journal: Journal) {
  editingId.value = journal.id;
  editForm.value = {
    name: journal.name,
    description: journal.description,
    tags: [...journal.tags],
  };
}

function cancelEdit() {
  editingId.value = null;
}

function saveEdit(journalId: string) {
  emit("update", journalId, { ...editForm.value });
  editingId.value = null;
}

function submitNew() {
  if (!newForm.value.name.trim()) return;
  emit("create", { ...newForm.value });
}

// ── TanStack columns ────────────────────────────────────────────────────────

const columns: ColumnDef<Journal, unknown>[] = [
  {
    accessorKey: "name",
    header: "Name",
    enableSorting: false,
  },
  {
    accessorKey: "description",
    header: "Description",
    enableSorting: false,
  },
  {
    accessorKey: "tags",
    header: "Tags",
    enableSorting: false,
  },
  {
    id: "actions",
    header: "Actions",
    enableSorting: false,
  },
];
</script>

<template>
  <AppDataTable :data="filteredJournals" :columns="columns">
    <!-- Header filter slots -->
    <template #header-name>
      <div class="py-1">
        <div class="text-xs font-medium mb-1">Name</div>
        <AppInput v-model="nameFilter" size="sm" placeholder="Filter name…" />
      </div>
    </template>

    <template #header-description>
      <div class="py-1">
        <div class="text-xs font-medium mb-1">Description</div>
        <AppInput
          v-model="descFilter"
          size="sm"
          placeholder="Filter description…"
        />
      </div>
    </template>

    <template #header-tags>
      <div class="py-1">
        <div class="text-xs font-medium mb-1">Tags</div>
        <AppInput v-model="tagsFilter" size="sm" placeholder="Filter tags…" />
      </div>
    </template>

    <template #header-actions>
      <div class="text-xs font-medium text-right py-1">Actions</div>
    </template>

    <!-- New row -->
    <template #body-prepend>
      <tr v-if="addingNew" class="border-b border-outline-variant">
        <td class="px-3 py-2">
          <AppInput
            v-model="newForm.name"
            size="sm"
            placeholder="Journal name"
            autofocus
            @keyup.enter="submitNew"
          />
        </td>
        <td class="px-3 py-2">
          <AppInput
            v-model="newForm.description"
            size="sm"
            placeholder="Description"
            @keyup.enter="submitNew"
          />
        </td>
        <td class="px-3 py-2">
          <AppTagInput
            v-model="newForm.tags"
            size="sm"
            placeholder="Add tag…"
            :suggestions="allTags"
          />
        </td>
        <td class="px-3 py-2">
          <div class="flex justify-end gap-1">
            <AppButton
              size="sm"
              variant="solid"
              :disabled="!newForm.name.trim()"
              @click="submitNew"
            >
              <AppIcon icon="lucide:check" size="sm" />
            </AppButton>
            <AppButton size="sm" variant="outlined" @click="emit('cancelNew')">
              <AppIcon icon="lucide:x" size="sm" />
            </AppButton>
          </div>
        </td>
      </tr>
    </template>

    <!-- Name cell -->
    <template #cell-name="{ row }">
      <AppInput
        v-if="editingId === row.original.id"
        v-model="editForm.name"
        size="sm"
      />
      <span v-else class="font-medium">{{ row.original.name }}</span>
    </template>

    <!-- Description cell -->
    <template #cell-description="{ row }">
      <AppInput
        v-if="editingId === row.original.id"
        v-model="editForm.description"
        size="sm"
      />
      <span v-else class="text-on-surface-variant">
        {{ row.original.description || "—" }}
      </span>
    </template>

    <!-- Tags cell -->
    <template #cell-tags="{ row }">
      <template v-if="editingId === row.original.id">
        <AppTagInput
          v-model="editForm.tags"
          size="sm"
          placeholder="Add tag…"
          :suggestions="allTags"
        />
      </template>
      <div
        v-else-if="row.original.tags.length"
        class="flex flex-wrap gap-1 py-1"
      >
        <AppChip
          v-for="tag in row.original.tags"
          :key="tag"
          variant="tonal"
          size="sm"
        >
          {{ tag }}
        </AppChip>
      </div>
      <span v-else class="text-on-surface-variant">—</span>
    </template>

    <!-- Actions cell -->
    <template #cell-actions="{ row }">
      <div class="flex justify-end gap-1">
        <template v-if="editingId === row.original.id">
          <AppButton
            size="sm"
            variant="solid"
            :disabled="!editForm.name.trim()"
            @click="saveEdit(row.original.id)"
          >
            <AppIcon icon="lucide:check" size="sm" />
          </AppButton>
          <AppButton size="sm" variant="outlined" @click="cancelEdit">
            <AppIcon icon="lucide:x" size="sm" />
          </AppButton>
        </template>
        <template v-else>
          <AppButton
            size="sm"
            variant="outlined"
            @click="startEdit(row.original)"
          >
            <AppIcon icon="lucide:pencil" size="sm" />
          </AppButton>
          <AppButton
            size="sm"
            variant="outlined"
            class="text-error border-error"
            @click="emit('delete', row.original)"
          >
            <AppIcon icon="lucide:trash-2" size="sm" />
          </AppButton>
        </template>
      </div>
    </template>

    <template #empty>
      <span class="text-on-surface-variant">No journals found.</span>
    </template>
  </AppDataTable>
</template>
