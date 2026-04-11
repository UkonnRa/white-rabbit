<script setup lang="ts">
import AppButton from "../components/ui/AppButton.vue";
import AppChip from "../components/ui/AppChip.vue";
import AppCard from "../components/ui/AppCard.vue";

const COLOR_ROLES = [
  { name: "Primary", bg: "bg-primary", fg: "text-on-primary" },
  { name: "On Primary", bg: "bg-on-primary", fg: "text-primary" },
  { name: "Secondary", bg: "bg-secondary", fg: "text-on-secondary" },
  { name: "On Secondary", bg: "bg-on-secondary", fg: "text-secondary" },
  { name: "Tertiary", bg: "bg-tertiary", fg: "text-on-tertiary" },
  { name: "On Tertiary", bg: "bg-on-tertiary", fg: "text-tertiary" },
  { name: "Error", bg: "bg-error", fg: "text-on-error" },
  { name: "On Error", bg: "bg-on-error", fg: "text-error" },
  { name: "Surface", bg: "bg-surface", fg: "text-on-surface" },
  {
    name: "Surface Variant",
    bg: "bg-surface-variant",
    fg: "text-on-surface-variant",
  },
  { name: "Background", bg: "bg-background", fg: "text-on-background" },
  { name: "Outline", bg: "bg-outline", fg: "text-on-primary" },
  { name: "Outline Variant", bg: "bg-outline-variant", fg: "text-on-surface" },
] as const;

const CONTAINER_ROLES = [
  {
    name: "Primary Container",
    bg: "bg-primary-container",
    fg: "text-on-primary-container",
  },
  {
    name: "Tertiary Container",
    bg: "bg-tertiary-container",
    fg: "text-on-surface",
  },
  {
    name: "Surface Container Lowest",
    bg: "bg-surface-container-lowest",
    fg: "text-on-surface",
  },
  {
    name: "Surface Container Low",
    bg: "bg-surface-container-low",
    fg: "text-on-surface",
  },
  {
    name: "Surface Container",
    bg: "bg-surface-container",
    fg: "text-on-surface",
  },
  {
    name: "Surface Container High",
    bg: "bg-surface-container-high",
    fg: "text-on-surface",
  },
  {
    name: "Surface Container Highest",
    bg: "bg-surface-container-highest",
    fg: "text-on-surface",
  },
  {
    name: "Inverse Surface",
    bg: "bg-inverse-surface",
    fg: "text-inverse-on-surface",
  },
] as const;
</script>

<template>
  <div class="flex flex-col gap-6">
    <h1 class="text-xl font-bold">Color Demo</h1>
    <p class="text-on-surface-variant text-sm">
      Switch themes and seed colors in the header. Toggle dark mode to see how
      <strong>dark mode is orthogonal to theme color</strong> — every seed color
      works in both light and dark.
    </p>

    <!-- Buttons -->
    <AppCard variant="outlined">
      <h2 class="text-lg font-semibold mb-4">Buttons</h2>
      <div class="flex flex-wrap gap-3">
        <AppButton variant="solid">Primary Solid</AppButton>
        <AppButton variant="outlined">Primary Outlined</AppButton>
        <AppButton variant="ghost">Primary Ghost</AppButton>
        <AppButton variant="text">Primary Text</AppButton>
      </div>
      <div class="flex flex-wrap gap-3 mt-3">
        <AppButton variant="solid" class="bg-secondary text-on-secondary"
          >Secondary</AppButton
        >
        <AppButton variant="solid" class="bg-tertiary text-on-tertiary"
          >Tertiary</AppButton
        >
        <AppButton variant="solid" class="bg-error text-on-error"
          >Error</AppButton
        >
      </div>
    </AppCard>

    <!-- Chips -->
    <AppCard variant="outlined">
      <h2 class="text-lg font-semibold mb-4">Chips</h2>
      <div class="flex flex-wrap gap-2">
        <AppChip variant="solid">Solid</AppChip>
        <AppChip variant="outlined">Outlined</AppChip>
        <AppChip variant="tonal">Tonal</AppChip>
        <AppChip variant="solid" closable>Closable</AppChip>
      </div>
    </AppCard>

    <!-- Core color roles -->
    <AppCard variant="outlined">
      <h2 class="text-lg font-semibold mb-4">Core Color Roles</h2>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        <div
          v-for="c in COLOR_ROLES"
          :key="c.name"
          :class="[c.bg, c.fg]"
          class="rounded-lg px-3 py-4 text-center text-sm font-medium"
        >
          {{ c.name }}
        </div>
      </div>
    </AppCard>

    <!-- Container / extension colors -->
    <AppCard variant="outlined">
      <h2 class="text-lg font-semibold mb-4">
        Container &amp; Extension Colors
      </h2>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        <div
          v-for="c in CONTAINER_ROLES"
          :key="c.name"
          :class="[c.bg, c.fg]"
          class="rounded-lg px-3 py-4 text-center text-sm font-medium"
        >
          {{ c.name }}
        </div>
      </div>
    </AppCard>

    <!-- Surface elevation demo -->
    <AppCard variant="outlined">
      <h2 class="text-lg font-semibold mb-4">Surface Hierarchy</h2>
      <div class="flex flex-wrap gap-3">
        <div
          class="bg-surface-container-lowest text-on-surface rounded-lg px-4 py-3 border border-outline-variant text-sm"
        >
          Lowest
        </div>
        <div
          class="bg-surface-container-low text-on-surface rounded-lg px-4 py-3 border border-outline-variant text-sm"
        >
          Low
        </div>
        <div
          class="bg-surface-container text-on-surface rounded-lg px-4 py-3 border border-outline-variant text-sm"
        >
          Container
        </div>
        <div
          class="bg-surface-container-high text-on-surface rounded-lg px-4 py-3 border border-outline-variant text-sm"
        >
          High
        </div>
        <div
          class="bg-surface-container-highest text-on-surface rounded-lg px-4 py-3 border border-outline-variant text-sm"
        >
          Highest
        </div>
      </div>
    </AppCard>
  </div>
</template>
