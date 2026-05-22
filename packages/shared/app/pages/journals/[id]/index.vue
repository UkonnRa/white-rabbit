<script setup lang="ts">
definePageMeta({ layout: "journal" });

const { journal, journalId } = useCurrentJournal();
const { data: accounts } = useAccounts(journalId);

const accountCount = computed(() => accounts.value?.length ?? 0);
</script>

<template>
  <div v-if="journal" class="flex flex-col gap-6">
    <div>
      <h1 class="text-xl font-bold">{{ journal.name }}</h1>
      <p
        v-if="journal.description"
        class="text-sm text-(--ui-text-dimmed) mt-1"
      >
        {{ journal.description }}
      </p>
      <div v-if="journal.tags?.length" class="flex flex-wrap gap-1 mt-2">
        <UBadge
          v-for="tag in journal.tags"
          :key="tag"
          variant="soft"
          size="sm"
        >
          {{ tag }}
        </UBadge>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <UCard
        class="cursor-pointer hover:border-(--ui-primary) transition-colors"
        @click="navigateTo(`/journals/${journalId}/accounts`)"
      >
        <div class="flex items-center gap-3">
          <UIcon
            icon="lucide:folder-tree"
            class="text-(--ui-primary) shrink-0 size-5"
          />
          <div>
            <h3 class="font-semibold">Accounts</h3>
            <p class="text-sm text-(--ui-text-dimmed)">
              {{ accountCount }} total
            </p>
          </div>
        </div>
      </UCard>

      <UCard class="opacity-40 select-none">
        <div class="flex items-center gap-3">
          <UIcon
            icon="lucide:list"
            class="text-(--ui-text-dimmed)/40 shrink-0 size-5"
          />
          <div>
            <h3 class="font-semibold">Records</h3>
            <p class="text-sm text-(--ui-text-dimmed)">Coming soon</p>
          </div>
        </div>
      </UCard>

      <UCard class="opacity-40 select-none">
        <div class="flex items-center gap-3">
          <UIcon
            icon="lucide:bar-chart-3"
            class="text-(--ui-text-dimmed)/40 shrink-0 size-5"
          />
          <div>
            <h3 class="font-semibold">Reports</h3>
            <p class="text-sm text-(--ui-text-dimmed)">Coming soon</p>
          </div>
        </div>
      </UCard>

      <UCard class="opacity-40 select-none">
        <div class="flex items-center gap-3">
          <UIcon
            icon="lucide:settings"
            class="text-(--ui-text-dimmed)/40 shrink-0 size-5"
          />
          <div>
            <h3 class="font-semibold">Settings</h3>
            <p class="text-sm text-(--ui-text-dimmed)">Coming soon</p>
          </div>
        </div>
      </UCard>
    </div>
  </div>
</template>
