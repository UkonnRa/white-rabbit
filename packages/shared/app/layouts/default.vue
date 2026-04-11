<script setup lang="ts">
import { ref } from "vue";
import { useMode } from "../composables/useMode";
import { useAppTheme } from "../composables/useAppTheme";
import AppButton from "../components/ui/AppButton.vue";
import AppIcon from "../components/ui/AppIcon.vue";

const { mode, toggle: toggleDark } = useMode();
const isDark = computed(() => mode.value === "dark");

const {
  themeName,
  seedColor,
  setTheme,
  setSeedColor,
  PALETTE,
  AVAILABLE_THEMES,
} = useAppTheme();

const drawerOpen = ref(false);
const showPalette = ref(false);
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
        <AppIcon icon="lucide:menu" size="md" />
      </AppButton>
      <h1 class="text-base font-semibold">White Rabbit</h1>

      <div class="ml-auto flex items-center gap-2">
        <!-- Theme switcher -->
        <div class="flex items-center gap-1">
          <AppButton
            v-for="t in AVAILABLE_THEMES"
            :key="t.name"
            :variant="themeName === t.name ? 'solid' : 'outlined'"
            size="sm"
            @click="setTheme(t.name)"
          >
            {{ t.label }}
          </AppButton>
        </div>

        <!-- Seed color picker -->
        <div class="relative">
          <AppButton
            variant="ghost"
            size="sm"
            aria-label="Change seed color"
            @click="showPalette = !showPalette"
          >
            <span
              class="inline-block size-5 rounded-full border border-outline-variant"
              :style="{ backgroundColor: seedColor }"
            />
            <AppIcon icon="lucide:palette" size="sm" />
          </AppButton>
          <div
            v-if="showPalette"
            class="absolute right-0 top-full mt-1 z-50 rounded-lg border border-outline-variant bg-surface shadow-lg p-2"
          >
            <div class="grid grid-cols-6 gap-2 p-1">
              <button
                v-for="color in PALETTE"
                :key="color.hex"
                type="button"
                class="size-6 rounded-full border-2 transition-transform hover:scale-110 cursor-pointer"
                :class="
                  seedColor === color.hex
                    ? 'border-primary ring-2 ring-primary/30'
                    : 'border-transparent'
                "
                :style="{ backgroundColor: color.hex }"
                :title="color.label"
                @click="
                  setSeedColor(color.hex);
                  showPalette = false;
                "
              />
            </div>
          </div>
        </div>

        <!-- Dark mode toggle -->
        <AppButton
          variant="ghost"
          size="sm"
          :aria-label="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="toggleDark"
        >
          <AppIcon :icon="isDark ? 'lucide:moon' : 'lucide:sun'" size="md" />
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
            <AppIcon icon="lucide:book-open" size="md" /> Journals
          </NuxtLink>
          <NuxtLink
            to="/color-demo"
            class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-on-surface hover:bg-surface-variant transition-colors"
          >
            <AppIcon icon="lucide:palette" size="md" /> Color Demo
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
