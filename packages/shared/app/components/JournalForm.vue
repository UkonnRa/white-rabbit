<script setup lang="ts">
import type { JournalFormData } from "~/models";

const props = defineProps<{
  initial?: { name: string; description: string; tags: string[] };
}>();

const emit = defineEmits<{
  submit: [data: JournalFormData];
  cancel: [];
}>();

const name = ref(props.initial?.name ?? "");
const description = ref(props.initial?.description ?? "");
const tags = ref<string[]>(props.initial?.tags ?? []);

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tags.value = val?.tags ?? [];
  },
);

function handleSubmit() {
  emit("submit", {
    name: name.value,
    description: description.value,
    tags: tags.value,
  });
}
</script>

<template>
  <v-card>
    <v-card-title class="pt-4 px-4">
      {{ initial ? "Edit Journal" : "New Journal" }}
    </v-card-title>
    <v-card-text>
      <form class="d-flex flex-column ga-4" @submit.prevent="handleSubmit">
        <v-text-field
          id="journal-name"
          v-model="name"
          label="Name"
          required
          placeholder="Journal name"
          variant="outlined"
          density="compact"
        />
        <v-textarea
          id="journal-description"
          v-model="description"
          label="Description"
          placeholder="Optional description"
          variant="outlined"
          density="compact"
          :rows="3"
        />
        <v-combobox
          id="journal-tags"
          v-model="tags"
          :delimiters="[',', ' ']"
          label="Tags"
          multiple
          chips
          closable-chips
          placeholder="tag1, tag2…"
          variant="outlined"
          density="compact"
        />
        <div class="d-flex justify-end ga-2">
          <v-btn
            type="button"
            variant="outlined"
            prepend-icon="mdi-close"
            @click="$emit('cancel')"
          >
            Cancel
          </v-btn>
          <v-btn type="submit" prepend-icon="mdi-content-save">
            {{ initial ? "Update" : "Create" }}
          </v-btn>
        </div>
      </form>
    </v-card-text>
  </v-card>
</template>
