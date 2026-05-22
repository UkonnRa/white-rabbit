<script setup lang="ts">
import { ref, computed } from "vue";
import type { Account } from "../models";

const props = defineProps<{
  account: Account;
  cascadeCount: number;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const confirmName = ref("");
const deleteEnabled = computed(() => confirmName.value === props.account.name);
</script>

<template>
  <div class="flex flex-col gap-4">
    <p class="text-sm">
      Delete account <strong>{{ account.name }}</strong>?
    </p>
    <p class="text-sm text-(--ui-text-dimmed)">
      This permanently deletes <strong>{{ account.name }}</strong> and
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}. Any records still
      posting to these accounts will fail to load. This action cannot be undone.
    </p>
    <div>
      <label for="delete-confirm" class="block text-sm mb-1">
        Type <strong>{{ account.name }}</strong> to confirm:
      </label>
      <UInput
        id="delete-confirm"
        v-model="confirmName"
        :placeholder="account.name"
      />
    </div>
    <div class="flex justify-end gap-2 pt-2">
      <UButton variant="outline" @click="emit('cancel')">Cancel</UButton>
      <UButton
        color="error"
        :disabled="!deleteEnabled"
        @click="emit('confirm')"
      >
        Delete
      </UButton>
    </div>
  </div>
</template>
