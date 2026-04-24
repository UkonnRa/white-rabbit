<script setup lang="ts">
import { h } from "vue";
import type { AccountRow, Account } from "../models";
import AppDataTable from "./ui/AppDataTable.vue";
import AppButton from "./ui/AppButton.vue";
import AppIcon from "./ui/AppIcon.vue";
import AppChip from "./ui/AppChip.vue";
import type { ColumnDef } from "@tanstack/vue-table";

const props = defineProps<{
  data: AccountRow[];
}>();

const emit = defineEmits<{
  create: [parentId: string];
  edit: [account: Account];
  archive: [account: Account];
  delete: [account: Account];
}>();

const TYPE_ICONS: Record<string, string> = {
  Asset: "lucide:landmark",
  Liability: "lucide:credit-card",
  Equity: "lucide:scale",
  Income: "lucide:trending-up",
  Expense: "lucide:trending-down",
};

const columns: ColumnDef<AccountRow, unknown>[] = [
  {
    id: "name",
    header: "Name",
    cell: ({ row }) => {
      const account = row.original;
      return h("div", { class: "flex items-center gap-2" }, [
        row.getCanExpand()
          ? h(AppButton, {
              variant: "ghost",
              size: "sm",
              "aria-label": account.name,
              onClick: row.getToggleExpandedHandler(),
            }, () => h(AppIcon, {
              icon: row.getIsExpanded() ? "lucide:chevron-down" : "lucide:chevron-right",
              size: "sm",
            }))
          : h("span", { class: "w-6 inline-block" }),

        h(AppIcon, {
          icon: TYPE_ICONS[account.type] ?? "lucide:folder",
          size: "sm",
        }),

        h("span", {
          class: account.parent_id === null ? "font-semibold text-on-surface" : "text-on-surface",
          style: { paddingLeft: `${account.depth * 12}px` },
        }, account.name),

        account.archived_at
          ? h("span", { class: "text-xs px-1.5 py-0.5 rounded bg-surface-variant text-on-surface-variant ml-2" }, "Archived")
          : null,
      ]);
    },
  },
  {
    id: "tags",
    header: "Tags",
    cell: ({ row }) => {
      const tags = row.original.tags;
      if (!tags.length) return null;
      return h("div", { class: "flex flex-wrap gap-1" },
        tags.map(tag => h(AppChip, { variant: "tonal", size: "sm" }, () => tag))
      );
    },
  },
  {
    id: "actions",
    header: "",
    cell: ({ row }) => {
      const account = row.original;
      const isRoot = account.parent_id === null;
      const isArchived = !!account.archived_at;

      const buttons: ReturnType<typeof h>[] = [];

      buttons.push(
        h(AppButton, {
          variant: "ghost", size: "sm",
          "aria-label": `Add child account under ${account.name}`,
          onClick: () => emit("create", account.id),
        }, () => h(AppIcon, { icon: "lucide:plus", size: "sm" }))
      );

      if (!isRoot) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            "aria-label": `Edit ${account.name}`,
            onClick: () => emit("edit", account as Account),
          }, () => h(AppIcon, { icon: "lucide:pencil", size: "sm" }))
        );
      }

      if (!isRoot && !isArchived) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            "aria-label": `Archive ${account.name}`,
            onClick: () => emit("archive", account as Account),
          }, () => h(AppIcon, { icon: "lucide:archive", size: "sm" }))
        );
      }

      if (!isRoot) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            class: "text-error",
            "aria-label": `Delete ${account.name}`,
            onClick: () => emit("delete", account as Account),
          }, () => h(AppIcon, { icon: "lucide:trash-2", size: "sm" }))
        );
      }

      return h("div", { class: "flex items-center gap-1" }, buttons);
    },
  },
];

function getSubRows(row: AccountRow): AccountRow[] {
  return row.subRows;
}
</script>

<template>
  <AppDataTable
    :data="data"
    :columns="columns"
    :get-sub-rows="getSubRows"
    :enable-expanding="true"
  >
    <template #empty>
      <div class="text-center py-8 text-on-surface-variant">
        No accounts found. Create one with the [+] button on a root account.
      </div>
    </template>
  </AppDataTable>
</template>
