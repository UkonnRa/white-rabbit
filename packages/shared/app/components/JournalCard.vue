<script setup lang="ts">
import type { Journal } from "../models";

defineProps<{
  journal: Journal;
}>();

defineEmits<{
  edit: [journal: Journal];
  delete: [journal: Journal];
}>();
</script>

<template>
  <v-card class="d-flex flex-column">
    <v-card-item>
      <div class="d-flex align-start justify-space-between ga-2">
        <v-card-title class="text-body-1 font-weight-medium">
          {{ journal.name }}
        </v-card-title>
        <div class="d-flex ga-1 shrink-0">
          <v-btn
            variant="outlined"
            size="small"
            @click="$emit('edit', journal)"
          >
            Edit
          </v-btn>
          <v-btn color="error" size="small" @click="$emit('delete', journal)">
            Delete
          </v-btn>
        </div>
      </div>
      <v-card-subtitle v-if="journal.description" class="px-0 mt-1">
        {{ journal.description }}
      </v-card-subtitle>
    </v-card-item>
    <v-card-text v-if="journal.tags.length" class="mt-auto pt-0">
      <div class="d-flex flex-wrap ga-1">
        <v-chip
          v-for="tag in journal.tags"
          :key="tag"
          size="small"
          variant="tonal"
        >
          {{ tag }}
        </v-chip>
      </div>
    </v-card-text>
  </v-card>
</template>
