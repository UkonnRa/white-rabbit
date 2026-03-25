<script setup lang="ts">
import type { Journal, JournalFormData } from "~/models";

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
    .sort((a, b) => {
      return (a.name || "").localeCompare(b.name || "");
    });
});

// ── Headers ─────────────────────────────────────────────────────────────────

const headers = [
  { title: "Name", key: "name", sortable: false },
  { title: "Description", key: "description", sortable: false },
  { title: "Tags", key: "tags", sortable: false },
  { title: "Actions", key: "actions", sortable: false, align: "end" as const },
];

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
</script>

<template>
  <v-data-table
    :items="filteredJournals"
    :headers="headers"
    :items-per-page="-1"
    item-value="id"
    density="compact"
    hide-default-footer
  >
    <!-- ── Header cells with filter inputs ─────────────────────────────── -->
    <template #header.name>
      <div class="py-1">
        <div class="text-subtitle-2 font-weight-medium mb-1">Name</div>
        <v-text-field
          v-model="nameFilter"
          placeholder="Filter name…"
          density="compact"
          variant="outlined"
          hide-details
          clearable
        />
      </div>
    </template>

    <template #header.description>
      <div class="py-1">
        <div class="text-subtitle-2 font-weight-medium mb-1">Description</div>
        <v-text-field
          v-model="descFilter"
          placeholder="Filter description…"
          density="compact"
          variant="outlined"
          hide-details
          clearable
        />
      </div>
    </template>

    <template #header.tags>
      <div class="py-1">
        <div class="text-subtitle-2 font-weight-medium mb-1">Tags</div>
        <v-text-field
          v-model="tagsFilter"
          placeholder="Filter tags…"
          density="compact"
          variant="outlined"
          hide-details
          clearable
        />
      </div>
    </template>

    <template #header.actions>
      <div class="text-subtitle-2 font-weight-medium text-right py-1">
        Actions
      </div>
    </template>

    <!-- ── New row ──────────────────────────────────────────────────────── -->
    <template #body.prepend>
      <tr v-if="addingNew">
        <td>
          <v-text-field
            v-model="newForm.name"
            placeholder="Journal name"
            density="compact"
            variant="outlined"
            hide-details
            autofocus
            @keyup.enter="submitNew"
          />
        </td>
        <td>
          <v-text-field
            v-model="newForm.description"
            placeholder="Description"
            density="compact"
            variant="outlined"
            hide-details
            @keyup.enter="submitNew"
          />
        </td>
        <td>
          <v-combobox
            v-model="newForm.tags"
            :delimiters="[',', ' ']"
            multiple
            chips
            closable-chips
            placeholder="tag1, tag2…"
            density="compact"
            variant="outlined"
            hide-details
          />
        </td>
        <td>
          <div class="d-flex justify-end ga-1">
            <v-btn
              icon
              size="small"
              color="secondary"
              :disabled="!newForm.name.trim()"
              @click="submitNew"
            >
              <v-icon>mdi-check</v-icon>
            </v-btn>
            <v-btn
              icon
              size="small"
              variant="outlined"
              @click="emit('cancelNew')"
            >
              <v-icon>mdi-close</v-icon>
            </v-btn>
          </div>
        </td>
      </tr>
    </template>

    <!-- ── Name cell ────────────────────────────────────────────────────── -->
    <template #item.name="{ item }">
      <v-text-field
        v-if="editingId === item.id"
        v-model="editForm.name"
        density="compact"
        variant="outlined"
        hide-details
      />
      <span v-else class="font-weight-medium">{{ item.name }}</span>
    </template>

    <!-- ── Description cell ─────────────────────────────────────────────── -->
    <template #item.description="{ item }">
      <v-text-field
        v-if="editingId === item.id"
        v-model="editForm.description"
        density="compact"
        variant="outlined"
        hide-details
      />
      <span v-else class="text-medium-emphasis">
        {{ item.description || "—" }}
      </span>
    </template>

    <!-- ── Tags cell ─────────────────────────────────────────────────────── -->
    <template #item.tags="{ item }">
      <v-combobox
        v-if="editingId === item.id"
        v-model="editForm.tags"
        :delimiters="[',']"
        multiple
        chips
        closable-chips
        density="compact"
        variant="outlined"
        hide-details
      />
      <div v-else-if="item.tags.length" class="d-flex flex-wrap ga-1 py-1">
        <v-chip
          v-for="tag in item.tags"
          :key="tag"
          size="small"
          variant="tonal"
        >
          {{ tag }}
        </v-chip>
      </div>
      <span v-else class="text-medium-emphasis">—</span>
    </template>

    <!-- ── Actions cell ──────────────────────────────────────────────────── -->
    <template #item.actions="{ item }">
      <div class="d-flex justify-end ga-1">
        <template v-if="editingId === item.id">
          <v-btn
            icon
            size="x-small"
            :disabled="!editForm.name.trim()"
            @click="saveEdit(item.id)"
          >
            <v-icon>mdi-check</v-icon>
          </v-btn>
          <v-btn icon size="small" variant="outlined" @click="cancelEdit">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </template>
        <template v-else>
          <v-btn
            color="secondary"
            icon
            size="x-small"
            variant="outlined"
            @click="startEdit(item)"
          >
            <v-icon>mdi-pencil</v-icon>
          </v-btn>
          <v-btn
            icon
            size="x-small"
            color="error"
            variant="outlined"
            @click="emit('delete', item)"
          >
            <v-icon>mdi-delete</v-icon>
          </v-btn>
        </template>
      </div>
    </template>

    <!-- ── Empty state ───────────────────────────────────────────────────── -->
    <template #no-data>
      <div class="text-center text-medium-emphasis py-6">
        No journals found.
      </div>
    </template>
  </v-data-table>
</template>
