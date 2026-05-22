<script setup lang="ts">
import { ref, watch } from "vue";
import type { JournalFormData } from "../models";

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
const tagInput = ref("");

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tags.value = val?.tags ?? [];
  },
);

function addTag() {
  const t = tagInput.value.trim();
  if (t && !tags.value.includes(t)) {
    tags.value.push(t);
  }
  tagInput.value = "";
}

function removeTag(tag: string) {
  tags.value = tags.value.filter((t) => t !== tag);
}

function handleSubmit() {
  emit("submit", {
    name: name.value,
    description: description.value,
    tags: tags.value,
  });
}
</script>

<template>
  <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
    <div>
      <label for="journal-name" class="block text-sm font-medium mb-1"
        >Name</label
      >
      <UInput
        id="journal-name"
        v-model="name"
        placeholder="Journal name"
        required
      />
    </div>
    <div>
      <label for="journal-description" class="block text-sm font-medium mb-1"
        >Description</label
      >
      <UTextarea
        id="journal-description"
        v-model="description"
        placeholder="Optional description"
        :rows="3"
      />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Tags</label>
      <div class="flex flex-wrap gap-1 mb-2">
        <UBadge
          v-for="tag in tags"
          :key="tag"
          variant="soft"
          size="sm"
          class="cursor-pointer"
          @click="removeTag(tag)"
        >
          {{ tag }} &times;
        </UBadge>
      </div>
      <form class="flex gap-2" @submit.prevent="addTag">
        <UInput v-model="tagInput" placeholder="Add tag..." size="sm" />
        <UButton type="submit" size="sm" variant="outline">Add</UButton>
      </form>
    </div>
    <div class="flex justify-end gap-2 pt-2">
      <UButton type="button" variant="outline" @click="$emit('cancel')">
        Cancel
      </UButton>
      <UButton type="submit">
        {{ initial ? "Update" : "Create" }}
      </UButton>
    </div>
  </form>
</template>
