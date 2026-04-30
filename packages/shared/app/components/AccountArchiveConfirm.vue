<script setup lang="ts">
import type { Account } from "../models";
import AppButton from "./ui/AppButton.vue";

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
    <p class="text-sm text-on-surface">
      Archive account <strong>{{ account.name }}</strong
      >?
    </p>
    <p class="text-sm text-on-surface-variant">
      This will also archive
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}. Archived accounts
      cannot receive new records, but their history is preserved.
    </p>
    <div class="flex justify-end gap-2 pt-2">
      <AppButton variant="outlined" @click="emit('cancel')">Cancel</AppButton>
      <AppButton variant="solid" @click="emit('confirm')">Archive</AppButton>
    </div>
  </div>
</template>
