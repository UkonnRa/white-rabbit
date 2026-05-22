<script setup lang="ts">
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <header
      class="sticky top-0 z-40 flex items-center gap-2 h-14 px-4 border-b border-(--ui-border) bg-(--ui-bg)"
    >
      <NuxtLink to="/" class="text-base font-semibold">
        White Rabbit
      </NuxtLink>
      <div class="ml-auto flex items-center gap-2">
        <UColorModeButton />
      </div>
    </header>

    <main class="flex-1 p-4 max-w-5xl mx-auto w-full">
      <slot />
    </main>

    <footer
      class="flex-none border-t border-(--ui-border) px-4 py-3 text-center text-sm text-(--ui-text-dimmed)"
    >
      {{ new Date().getFullYear() }} — <strong>White Rabbit</strong>, Ukonn Ra
    </footer>
  </div>
</template>
