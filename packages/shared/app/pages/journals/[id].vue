<script setup lang="ts">
import AppCard from "../../components/ui/AppCard.vue";
import AppChip from "../../components/ui/AppChip.vue";
import AppIcon from "../../components/ui/AppIcon.vue";

definePageMeta({ layout: "journal" });

const { journal, journalId } = useCurrentJournal();
const { data: accounts } = useAccounts(journalId);

const accountCount = computed(() => accounts.value?.length ?? 0);
</script>

<template>
  <div v-if="journal" class="flex flex-col gap-6">
    <div>
      <h1 class="text-xl font-bold text-on-background">{{ journal.name }}</h1>
      <p v-if="journal.description" class="text-sm text-on-surface-variant mt-1">
        {{ journal.description }}
      </p>
      <div v-if="journal.tags?.length" class="flex flex-wrap gap-1 mt-2">
        <AppChip v-for="tag in journal.tags" :key="tag" variant="tonal" size="sm">
          {{ tag }}
        </AppChip>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <AppCard
        variant="outlined"
        class="p-4 cursor-pointer hover:border-primary transition-colors"
        @click="navigateTo(`/journals/${journalId}/accounts`)"
      >
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:folder-tree" size="md" class="text-primary shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Accounts</h3>
            <p class="text-sm text-on-surface-variant">
              {{ accountCount }} total
            </p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:list" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Records</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:bar-chart-3" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Reports</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:settings" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Settings</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
