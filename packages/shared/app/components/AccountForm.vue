<script setup lang="ts">
import { ref, watch } from "vue";
import type { AccountFormData } from "../models";
import AppInput from "./ui/AppInput.vue";
import AppTextarea from "./ui/AppTextarea.vue";
import AppButton from "./ui/AppButton.vue";
import AppTagInput from "./ui/AppTagInput.vue";

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
const error = ref<string | null>(null);

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tags.value = val?.tags ?? [];
  },
);

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
    <div v-if="parentPath" class="text-sm text-on-surface-variant">
      Parent: <span class="font-medium">{{ parentPath }}</span>
    </div>

    <div>
      <label for="account-name" class="block text-sm font-medium mb-1"
        >Name</label
      >
      <AppInput
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
      <AppTextarea
        id="account-description"
        v-model="description"
        placeholder="Optional description"
        :rows="3"
      />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Tags</label>
      <AppTagInput v-model="tags" placeholder="Add tag…" />
    </div>

    <div v-if="error" class="text-error text-sm">{{ error }}</div>

    <div class="flex justify-end gap-2 pt-2">
      <AppButton type="button" variant="outlined" @click="$emit('cancel')">
        Cancel
      </AppButton>
      <AppButton type="submit" variant="solid">
        {{ initial ? "Update" : "Create" }}
      </AppButton>
    </div>
  </form>
</template>
