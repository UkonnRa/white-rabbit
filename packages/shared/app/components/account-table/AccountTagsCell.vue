<script setup lang="ts">
import type { AccountRow } from "../../models";

defineProps<{
  account: AccountRow;
}>();
</script>

<template>
  <div v-if="account.tags.length" class="flex flex-wrap gap-1">
    <UBadge v-for="tag in account.tags" :key="tag" variant="soft" size="sm">
      {{ tag }}
    </UBadge>
  </div>
</template>
