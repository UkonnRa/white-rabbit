<script setup lang="ts">
import { computed } from "vue";
import type { ExpandedState, Row } from "@tanstack/vue-table";
import type { AccountRow } from "../../models";
import AppButton from "../ui/AppButton.vue";
import AppIcon from "../ui/AppIcon.vue";

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
    <AppButton
      v-if="canExpand"
      variant="ghost"
      size="sm"
      :aria-label="account.name"
      @click="emit('toggleExpand', row.id)"
    >
      <AppIcon
        :icon="isExpanded ? 'lucide:chevron-down' : 'lucide:chevron-right'"
        size="sm"
      />
    </AppButton>
    <span v-else class="w-6 inline-block" />

    <AppIcon :icon="TYPE_ICONS[account.type] ?? 'lucide:folder'" size="sm" />

    <span
      :class="
        account.parentId === null
          ? 'font-semibold text-on-surface'
          : 'text-on-surface'
      "
      :style="{ paddingLeft: `${account.depth * 12}px` }"
    >
      {{ account.name }}
    </span>

    <span
      v-if="account.archivedAt"
      class="text-xs px-1.5 py-0.5 rounded bg-surface-variant text-on-surface-variant ml-2"
    >
      Archived
    </span>
  </div>
</template>
