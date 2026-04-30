<script setup lang="ts">
import type { AccountRow } from "../../models";
import AppChip from "../ui/AppChip.vue";

defineProps<{
  account: AccountRow;
}>();
</script>

<template>
  <div v-if="account.tags.length" class="flex flex-wrap gap-1">
    <AppChip v-for="tag in account.tags" :key="tag" variant="tonal" size="sm">
      {{ tag }}
    </AppChip>
  </div>
</template>
