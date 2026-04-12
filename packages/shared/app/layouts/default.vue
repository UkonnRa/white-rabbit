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

const showPalette = ref(false);
</script>

<template>
  <div class="min-h-screen flex flex-col bg-background text-on-background">
    <!-- App header -->
    <header
      class="sticky top-0 z-40 flex items-center gap-2 h-14 px-4 border-b border-outline-variant bg-surface"
    >
      <NuxtLink to="/" class="text-base font-semibold text-on-surface">
        White Rabbit
      </NuxtLink>

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
            <!-- Secret color demo link -->
            <NuxtLink
              to="/color-demo"
              class="block mt-2 pt-2 border-t border-outline-variant text-xs text-on-surface-variant/50 hover:text-primary text-center transition-colors"
              @click="showPalette = false"
            >
              Color reference
            </NuxtLink>
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

    <!-- Main content -->
    <main class="flex-1 p-4 max-w-5xl mx-auto w-full">
      <slot />
    </main>

    <!-- Footer -->
    <footer
      class="flex-none border-t border-outline-variant px-4 py-3 text-center text-sm text-on-surface-variant"
    >
      {{ new Date().getFullYear() }} — <strong>White Rabbit</strong>, Ukonn Ra
    </footer>
  </div>
</template>
