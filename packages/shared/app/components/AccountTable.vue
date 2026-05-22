<script setup lang="ts">
import { h, ref } from "vue";
import type { ExpandedState } from "@tanstack/vue-table";
import type { AccountRow, Account, AccountType } from "../models";
import AccountNameCell from "./account-table/AccountNameCell.vue";
import AccountTagsCell from "./account-table/AccountTagsCell.vue";
import AccountActionsCell from "./account-table/AccountActionsCell.vue";
import type { ColumnDef } from "@tanstack/vue-table";

defineProps<{
  data: AccountRow[];
}>();

const emit = defineEmits<{
  create: [parentId: string, type: AccountType];
  edit: [account: Account];
  archive: [account: Account];
  delete: [account: Account];
}>();

const expanded = ref<ExpandedState>({});

const columns: ColumnDef<AccountRow, unknown>[] = [
  {
    id: "name",
    header: "Name",
    cell: ({ row }) =>
      h(AccountNameCell, {
        account: row.original,
        row,
        expanded: expanded.value,
        onToggleExpand: (rowId: string) => {
          const next: Record<string, boolean> = {
            ...(expanded.value as Record<string, boolean>),
          };
          if (next[rowId]) {
            delete next[rowId];
          } else {
            next[rowId] = true;
          }
          expanded.value = next;
        },
      }),
  },
  {
    id: "tags",
    header: "Tags",
    cell: ({ row }) => h(AccountTagsCell, { account: row.original }),
  },
  {
    id: "actions",
    header: "Actions",
    cell: ({ row }) =>
      h(AccountActionsCell, {
        account: row.original,
        onCreate: (parentId: string, type: string) =>
          emit("create", parentId, type as AccountType),
        onEdit: (account: Account) => emit("edit", account),
        onArchive: (account: Account) => emit("archive", account),
        onDelete: (account: Account) => emit("delete", account),
      }),
  },
];

function getSubRows(row: AccountRow): AccountRow[] {
  return row.subRows;
}
</script>

<template>
  <UTable
    :data="data"
    :columns="columns"
    :get-sub-rows="getSubRows"
    :enable-expanding="true"
    :expanded="expanded"
    @update:expanded="expanded = $event"
  >
    <template #empty>
      <div class="text-center py-8 text-(--ui-text-dimmed)">
        No accounts found. Create one with the [+] button on a root account.
      </div>
    </template>
  </UTable>
</template>
