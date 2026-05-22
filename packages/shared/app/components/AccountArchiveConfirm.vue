<script setup lang="ts">
import type { Account } from "../models";

defineProps<{
  account: Account;
  cascadeCount: number;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <div class="flex flex-col gap-4">
    <p class="text-sm">
      Archive account <strong>{{ account.name }}</strong>?
    </p>
    <p class="text-sm text-(--ui-text-dimmed)">
      This will also archive
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}. Archived accounts
      cannot receive new records, but their history is preserved.
    </p>
    <div class="flex justify-end gap-2 pt-2">
      <UButton variant="outline" @click="emit('cancel')">Cancel</UButton>
      <UButton @click="emit('confirm')">Archive</UButton>
    </div>
  </div>
</template>
