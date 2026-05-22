<script setup lang="ts">
import type { AccountRow, Account } from "../../models";

const props = defineProps<{
  account: AccountRow;
}>();

const emit = defineEmits<{
  create: [parentId: string, type: string];
  edit: [account: Account];
  archive: [account: Account];
  delete: [account: Account];
}>();

const isRoot = props.account.parentId === null;
const isArchived = !!props.account.archivedAt;
</script>

<template>
  <div class="flex items-center gap-1">
    <UButton
      variant="ghost"
      size="sm"
      icon="lucide:plus"
      title="Add child account"
      :aria-label="`Add child account under ${account.name}`"
      @click="emit('create', account.id, account.type)"
    />

    <UButton
      v-if="!isRoot"
      variant="ghost"
      size="sm"
      icon="lucide:pencil"
      title="Edit account"
      :aria-label="`Edit ${account.name}`"
      @click="emit('edit', account as Account)"
    />

    <UButton
      v-if="!isRoot && !isArchived"
      variant="ghost"
      size="sm"
      icon="lucide:archive"
      title="Archive account"
      :aria-label="`Archive ${account.name}`"
      @click="emit('archive', account as Account)"
    />

    <UButton
      v-if="!isRoot"
      variant="ghost"
      size="sm"
      icon="lucide:trash-2"
      color="error"
      title="Delete account"
      :aria-label="`Delete ${account.name}`"
      @click="emit('delete', account as Account)"
    />
  </div>
</template>
