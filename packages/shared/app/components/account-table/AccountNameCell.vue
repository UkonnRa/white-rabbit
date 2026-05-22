<script setup lang="ts">
import { computed } from "vue";
import type { ExpandedState, Row } from "@tanstack/vue-table";
import type { AccountRow } from "../../models";

const TYPE_ICONS: Record<string, string> = {
  Asset: "lucide:landmark",
  Liability: "lucide:credit-card",
  Equity: "lucide:scale",
  Income: "lucide:trending-up",
  Expense: "lucide:trending-down",
};

const props = defineProps<{
  account: AccountRow;
  row: Row<AccountRow>;
  expanded: ExpandedState;
}>();

const emit = defineEmits<{
  toggleExpand: [rowId: string];
}>();

const canExpand = computed(() => props.row.getCanExpand());
const isExpanded = computed(() => {
  if (typeof props.expanded === "object" && props.expanded !== null) {
    return !!props.expanded[props.row.id];
  }
  return false;
});
</script>

<template>
  <div class="flex items-center gap-2">
    <UButton
      v-if="canExpand"
      variant="ghost"
      size="sm"
      :icon="isExpanded ? 'lucide:chevron-down' : 'lucide:chevron-right'"
      :aria-label="account.name"
      @click="emit('toggleExpand', row.id)"
    />
    <span v-else class="w-6 inline-block" />

    <UIcon :name="TYPE_ICONS[account.type] ?? 'lucide:folder'" size="sm" />

    <span
      :class="account.parentId === null ? 'font-semibold' : ''"
      :style="{ paddingLeft: `${account.depth * 12}px` }"
    >
      {{ account.name }}
    </span>

    <span
      v-if="account.archivedAt"
      class="text-xs px-1.5 py-0.5 rounded bg-(--ui-bg-elevated) text-(--ui-text-dimmed) ml-2"
    >
      Archived
    </span>
  </div>
</template>
