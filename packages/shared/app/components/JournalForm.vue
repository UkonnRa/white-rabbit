<script setup lang="ts">
import { ref, watch } from "vue";
import type { JournalFormData } from "../models";
import AppCard from "./ui/AppCard.vue";
import AppInput from "./ui/AppInput.vue";
import AppTextarea from "./ui/AppTextarea.vue";
import AppButton from "./ui/AppButton.vue";

const props = defineProps<{
  initial?: { name: string; description: string; tags: string[] };
}>();

const emit = defineEmits<{
  submit: [data: JournalFormData];
  cancel: [];
}>();

const name = ref(props.initial?.name ?? "");
const description = ref(props.initial?.description ?? "");
const tagInput = ref("");
const tags = ref<string[]>(props.initial?.tags ?? []);

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tags.value = val?.tags ?? [];
  },
);

function addTag() {
  const raw = tagInput.value.trim();
  if (raw) {
    for (const t of raw.split(/[, ]+/)) {
      const trimmed = t.trim();
      if (trimmed && !tags.value.includes(trimmed)) {
        tags.value.push(trimmed);
      }
    }
    tagInput.value = "";
  }
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
  <AppCard variant="outlined">
    <h2 class="text-lg font-semibold mb-4">
      {{ initial ? "Edit Journal" : "New Journal" }}
    </h2>
    <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
      <div>
        <label for="journal-name" class="block text-sm font-medium mb-1"
          >Name</label
        >
        <AppInput
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
        <AppTextarea
          id="journal-description"
          v-model="description"
          placeholder="Optional description"
          :rows="3"
        />
      </div>
      <div>
        <label for="journal-tags" class="block text-sm font-medium mb-1"
          >Tags</label
        >
        <AppInput
          id="journal-tags"
          v-model="tagInput"
          placeholder="Type a tag and press Enter…"
          @keydown.enter.prevent="addTag"
          @blur="addTag"
        />
        <div v-if="tags.length" class="flex flex-wrap gap-1 mt-2">
          <span
            v-for="tag in tags"
            :key="tag"
            class="inline-flex items-center gap-1 rounded-full bg-surface-variant text-on-surface-variant px-2 py-0.5 text-xs"
          >
            {{ tag }}
            <button
              type="button"
              class="ml-0.5 hover:text-error cursor-pointer"
              aria-label="Remove tag"
              @click="removeTag(tag)"
            >
              &#x2715;
            </button>
          </span>
        </div>
      </div>
      <div class="flex justify-end gap-2">
        <AppButton type="button" variant="outlined" @click="$emit('cancel')">
          Cancel
        </AppButton>
        <AppButton type="submit" variant="solid">
          {{ initial ? "Update" : "Create" }}
        </AppButton>
      </div>
    </form>
  </AppCard>
</template>
