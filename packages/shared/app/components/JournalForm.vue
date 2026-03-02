<script setup lang="ts">
import type { JournalFormData } from "../../models";

const props = defineProps<{
  initial?: { name: string; description: string; tags: string[] };
}>();

const emit = defineEmits<{
  submit: [data: JournalFormData];
  cancel: [];
}>();

const name = ref(props.initial?.name ?? "");
const description = ref(props.initial?.description ?? "");
const tagsInput = ref(props.initial?.tags?.join(", ") ?? "");

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tagsInput.value = val?.tags?.join(", ") ?? "";
  },
);

function handleSubmit() {
  const tags = tagsInput.value
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t.length > 0);

  emit("submit", {
    name: name.value,
    description: description.value,
    tags,
  });
}
</script>

<template>
  <form class="journal-form" @submit.prevent="handleSubmit">
    <div class="field">
      <label for="journal-name">Name</label>
      <input
        id="journal-name"
        v-model="name"
        type="text"
        required
        placeholder="Journal name"
      />
    </div>
    <div class="field">
      <label for="journal-description">Description</label>
      <textarea
        id="journal-description"
        v-model="description"
        placeholder="Optional description"
        rows="3"
      />
    </div>
    <div class="field">
      <label for="journal-tags">Tags</label>
      <input
        id="journal-tags"
        v-model="tagsInput"
        type="text"
        placeholder="Comma-separated tags"
      />
    </div>
    <div class="actions">
      <button type="submit">{{ initial ? "Update" : "Create" }}</button>
      <button type="button" @click="$emit('cancel')">Cancel</button>
    </div>
  </form>
</template>
