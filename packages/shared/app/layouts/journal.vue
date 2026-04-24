<script setup lang="ts">
const route = useRoute();
const journalId = computed(() => route.params.id as string);
const { data: journal } = useJournal(journalId);

const menuItems = computed(() => [
  { label: "Dashboard", to: `/journals/${journalId.value}`, enabled: true },
  { label: "Accounts", to: `/journals/${journalId.value}/accounts`, enabled: true },
  { label: "Records", to: "", enabled: false },
  { label: "Reports", to: "", enabled: false },
]);

function isActive(path: string) {
  return route.path === path;
}
</script>

<template>
  <NuxtLayout name="default">
    <div class="flex">
      <aside class="w-48 shrink-0 border-r border-outline-variant p-3">
        <nav class="flex flex-col gap-1">
          <template v-for="item in menuItems" :key="item.label">
            <NuxtLink
              v-if="item.enabled"
              :to="item.to"
              class="block px-3 py-2 rounded text-sm transition-colors"
              :class="isActive(item.to)
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-on-surface-variant hover:bg-surface-variant'"
            >
              {{ item.label }}
            </NuxtLink>
            <span
              v-else
              class="block px-3 py-2 rounded text-sm text-on-surface-variant/30 cursor-not-allowed select-none"
            >
              {{ item.label }}
            </span>
          </template>
        </nav>
      </aside>
      <main class="flex-1 p-4 min-w-0">
        <slot />
      </main>
    </div>
  </NuxtLayout>
</template>
