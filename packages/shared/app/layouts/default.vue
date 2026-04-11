<script setup lang="ts">
import { ref, computed } from "vue";
import { useMode } from "../composables/useMode";
import AppButton from "../components/ui/AppButton.vue";

const { mode, toggle: toggleDark } = useMode();
const isDark = computed(() => mode.value === "dark");

const drawerOpen = ref(false);
</script>

<template>
  <div class="min-h-screen flex flex-col bg-background text-on-background">
    <!-- App bar -->
    <header
      class="sticky top-0 z-40 flex items-center gap-2 h-14 px-4 border-b border-outline-variant bg-surface"
    >
      <AppButton
        variant="ghost"
        size="sm"
        aria-label="Toggle navigation"
        @click="drawerOpen = !drawerOpen"
      >
        &#9776;
      </AppButton>
      <h1 class="text-base font-semibold">White Rabbit</h1>

      <div class="ml-auto flex items-center gap-1">
        <AppButton
          variant="ghost"
          size="sm"
          :aria-label="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="toggleDark"
        >
          {{ isDark ? "&#x1F319;" : "&#x2600;&#xFE0F;" }}
        </AppButton>
      </div>
    </header>

    <div class="flex flex-1">
      <!-- Navigation drawer -->
      <aside
        v-if="drawerOpen"
        class="w-60 shrink-0 border-r border-outline-variant bg-surface p-2"
      >
        <nav>
          <NuxtLink
            to="/"
            class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-on-surface hover:bg-surface-variant transition-colors"
          >
            &#x1F4D6; Journals
          </NuxtLink>
        </nav>
      </aside>

      <!-- Main content -->
      <main class="flex-1 p-4 max-w-5xl mx-auto w-full">
        <slot />
      </main>
    </div>

    <!-- Footer -->
    <footer
      class="flex-none border-t border-outline-variant px-4 py-3 text-center text-sm text-on-surface-variant"
    >
      {{ new Date().getFullYear() }} — <strong>White Rabbit</strong>, Ukonn Ra
    </footer>
  </div>
</template>
