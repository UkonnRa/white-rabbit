<script setup lang="ts">
import { ref } from "vue";
import type { JournalFilter } from "../models";

const emit = defineEmits<{
  search: [filter: JournalFilter];
}>();

const name = ref("");
const tag = ref("");
const fullText = ref("");

function handleSearch() {
  const filter: JournalFilter = {};
  if (name.value.trim()) filter.name = name.value.trim();
  if (tag.value.trim()) filter.tag = tag.value.trim();
  if (fullText.value.trim()) filter.fullText = fullText.value.trim();
  emit("search", filter);
}

function handleClear() {
  name.value = "";
  tag.value = "";
  fullText.value = "";
  emit("search", {});
}
</script>

<template>
  <form class="journal-filter" @submit.prevent="handleSearch">
    <input v-model="fullText" type="text" placeholder="Search..." />
    <input v-model="name" type="text" placeholder="Filter by name" />
    <input v-model="tag" type="text" placeholder="Filter by tag" />
    <button type="submit">Search</button>
    <button type="button" @click="handleClear">Clear</button>
  </form>
</template>
