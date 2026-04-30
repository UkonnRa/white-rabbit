<script setup lang="ts">
import { ref, computed } from "vue";
import type { Account } from "../models";
import AppButton from "./ui/AppButton.vue";
import AppInput from "./ui/AppInput.vue";

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
    <p class="text-sm text-on-surface">
      Delete account <strong>{{ account.name }}</strong
      >?
    </p>
    <p class="text-sm text-on-surface-variant">
      This permanently deletes <strong>{{ account.name }}</strong> and
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}. Any records still
      posting to these accounts will fail to load. This action cannot be undone.
    </p>
    <div>
      <label for="delete-confirm" class="block text-sm mb-1">
        Type <strong>{{ account.name }}</strong> to confirm:
      </label>
      <AppInput
        id="delete-confirm"
        v-model="confirmName"
        :placeholder="account.name"
      />
    </div>
    <div class="flex justify-end gap-2 pt-2">
      <AppButton variant="outlined" @click="emit('cancel')">Cancel</AppButton>
      <AppButton
        variant="solid"
        class="bg-error text-on-error"
        :disabled="!deleteEnabled"
        @click="emit('confirm')"
      >
        Delete
      </AppButton>
    </div>
  </div>
</template>
