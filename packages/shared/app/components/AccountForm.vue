<script setup lang="ts">
import { ref, watch } from "vue";
import type { AccountFormData } from "../models";

const props = defineProps<{
  initial?: { name: string; description: string; tags: string[] };
  parentPath?: string;
}>();

const emit = defineEmits<{
  submit: [data: AccountFormData];
  cancel: [];
}>();

const RESERVED_NAMES = ["Asset", "Liability", "Equity", "Income", "Expense"];

const name = ref(props.initial?.name ?? "");
const description = ref(props.initial?.description ?? "");
const tags = ref<string[]>(props.initial?.tags ?? []);
const tagInput = ref("");
const error = ref<string | null>(null);

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

function validate(): boolean {
  if (!name.value.trim()) {
    error.value = "Name is required.";
    return false;
  }
  if (
    RESERVED_NAMES.some(
      (r) =>
        r.localeCompare(name.value, undefined, { sensitivity: "base" }) === 0,
    )
  ) {
    error.value = `"${name.value}" is a reserved root account name.`;
    return false;
  }
  error.value = null;
  return true;
}

function handleSubmit() {
  if (!validate()) return;
  emit("submit", {
    name: name.value,
    description: description.value,
    tags: tags.value,
  });
}
</script>

<template>
  <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
    <div v-if="parentPath" class="text-sm text-(--ui-text-dimmed)">
      Parent: <span class="font-medium">{{ parentPath }}</span>
    </div>

    <div>
      <label for="account-name" class="block text-sm font-medium mb-1"
        >Name</label
      >
      <UInput
        id="account-name"
        v-model="name"
        placeholder="Account name"
        required
      />
    </div>
    <div>
      <label for="account-description" class="block text-sm font-medium mb-1"
        >Description</label
      >
      <UTextarea
        id="account-description"
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

    <div v-if="error" class="text-(--ui-error) text-sm">{{ error }}</div>

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
