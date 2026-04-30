<script setup lang="ts">
import type { AccountRow, Account } from "../../models";
import AppButton from "../ui/AppButton.vue";
import AppIcon from "../ui/AppIcon.vue";

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
    <AppButton
      variant="ghost"
      size="sm"
      title="Add child account"
      :aria-label="`Add child account under ${account.name}`"
      @click="emit('create', account.id, account.type)"
    >
      <AppIcon icon="lucide:plus" size="sm" />
    </AppButton>

    <AppButton
      v-if="!isRoot"
      variant="ghost"
      size="sm"
      title="Edit account"
      :aria-label="`Edit ${account.name}`"
      @click="emit('edit', account as Account)"
    >
      <AppIcon icon="lucide:pencil" size="sm" />
    </AppButton>

    <AppButton
      v-if="!isRoot && !isArchived"
      variant="ghost"
      size="sm"
      title="Archive account"
      :aria-label="`Archive ${account.name}`"
      @click="emit('archive', account as Account)"
    >
      <AppIcon icon="lucide:archive" size="sm" />
    </AppButton>

    <AppButton
      v-if="!isRoot"
      variant="ghost"
      size="sm"
      class="text-error"
      title="Delete account"
      :aria-label="`Delete ${account.name}`"
      @click="emit('delete', account as Account)"
    >
      <AppIcon icon="lucide:trash-2" size="sm" />
    </AppButton>
  </div>
</template>
