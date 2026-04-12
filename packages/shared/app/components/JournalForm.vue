<script setup lang="ts">
import { ref, watch } from "vue";
import type { JournalFormData } from "../models";
import AppInput from "./ui/AppInput.vue";
import AppTextarea from "./ui/AppTextarea.vue";
import AppButton from "./ui/AppButton.vue";
import AppTagInput from "./ui/AppTagInput.vue";

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
      <label class="block text-sm font-medium mb-1">Tags</label>
      <AppTagInput v-model="tags" placeholder="Add tag…" />
    </div>
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
