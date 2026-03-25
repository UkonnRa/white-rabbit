<script setup lang="ts">
import { useTheme } from "vuetify";

const theme = useTheme();
const isDark = computed(() => theme.global.current.value.dark);
function toggleDark() {
  theme.global.name.value = isDark.value ? "light" : "dark";
}

const drawer = ref<boolean | null>(null);

const { seed, applySeed, PALETTE } = useAppTheme();
</script>

<template>
  <v-app-bar>
    <v-app-bar-nav-icon @click="drawer = !drawer" />
    <v-app-bar-title>White Rabbit</v-app-bar-title>

    <template #append>
      <!-- Theme color picker -->
      <v-menu :close-on-content-click="false" max-width="220">
        <template #activator="{ props }">
          <v-btn icon variant="text" v-bind="props">
            <v-icon>mdi-palette</v-icon>
          </v-btn>
        </template>
        <v-card>
          <v-card-title class="text-body-2 pa-3 pb-1">Theme Color</v-card-title>
          <v-list density="compact" nav>
            <v-list-item
              v-for="color in PALETTE"
              :key="color.hex"
              :title="color.label"
              :active="seed === color.hex"
              rounded="lg"
              @click="applySeed(color.hex)"
            >
              <template #prepend>
                <v-avatar :color="color.hex" size="20" />
              </template>
              <template #append>
                <v-icon v-if="seed === color.hex" size="16">mdi-check</v-icon>
              </template>
            </v-list-item>
          </v-list>
        </v-card>
      </v-menu>

      <!-- Dark mode toggle -->
      <v-btn icon variant="text" @click="toggleDark">
        <v-icon>{{
          isDark ? "mdi-weather-night" : "mdi-white-balance-sunny"
        }}</v-icon>
      </v-btn>
    </template>
  </v-app-bar>

  <v-navigation-drawer v-model="drawer">
    <v-list nav density="compact">
      <v-list-item
        prepend-icon="mdi-book-open-variant"
        title="Journals"
        to="/"
        rounded="lg"
      />
    </v-list>
  </v-navigation-drawer>

  <v-main>
    <v-container>
      <slot />
    </v-container>
  </v-main>

  <v-footer border color="tertiary" class="flex-0">
    <div class="flex-1-0-100 text-center">
      {{ new Date().getFullYear() }} — <strong>White Rabbit</strong>, Ukonn Ra
    </div>
  </v-footer>
</template>
