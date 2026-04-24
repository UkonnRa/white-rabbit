<script setup lang="ts">
import { ref, computed } from "vue";
import type { Account, AccountFormData } from "../../../models";
import AppButton from "../../../components/ui/AppButton.vue";
import AppIcon from "../../../components/ui/AppIcon.vue";
import AppInput from "../../../components/ui/AppInput.vue";
import AppDialog from "../../../components/ui/AppDialog.vue";
import AccountTable from "../../../components/AccountTable.vue";
import AccountForm from "../../../components/AccountForm.vue";
import AccountArchiveConfirm from "../../../components/AccountArchiveConfirm.vue";
import AccountDeleteConfirm from "../../../components/AccountDeleteConfirm.vue";

definePageMeta({ layout: "journal" });

const { journalId } = useCurrentJournal();
const { data: accounts, status, refresh: refreshAccounts } = useAccounts(journalId);

const showArchived = ref(false);

const { tree } = useAccountTree(
  filteredAccounts,
  showArchived,
);

const client = useAccountClient();
const mutationError = ref<string | null>(null);

const searchQuery = ref("");

const filteredAccounts = computed(() => {
  const list = accounts.value ?? [];
  const q = searchQuery.value.toLowerCase().trim();
  if (!q) return list;
  return list.filter(a =>
    a.name.toLowerCase().includes(q) ||
    a.tags.some(t => t.toLowerCase().includes(q))
  );
});

const showCreateDialog = ref(false);
const creatingParentId = ref<string | null>(null);
const creatingParentPath = ref("");
const creatingParentType = ref("");

function getParentPath(accountId: string): string {
  const find = (rows: typeof tree.value): string | null => {
    for (const r of rows) {
      if (r.id === accountId) return r.name;
      const child = find(r.subRows);
      if (child) return `${r.name} > ${child}`;
    }
    return null;
  };
  return find(tree.value) ?? "";
}

function openCreate(parentId: string, parentType: string) {
  creatingParentId.value = parentId;
  creatingParentPath.value = getParentPath(parentId);
  creatingParentType.value = parentType;
  showCreateDialog.value = true;
  mutationError.value = null;
}

async function handleCreate(data: AccountFormData) {
  if (!creatingParentId.value) return;
  mutationError.value = null;
  try {
    await client.create({
      journal_id: journalId.value,
      parent_id: creatingParentId.value,
      type: creatingParentType.value,
      ...data,
    });
    showCreateDialog.value = false;
    await refreshAccounts();
  } catch (e) {
    mutationError.value = String(e);
  }
}

const showEditDialog = ref(false);
const editingAccount = ref<Account | null>(null);

async function openEdit(account: Account) {
  editingAccount.value = account;
  showEditDialog.value = true;
  mutationError.value = null;
}

async function handleUpdate(data: AccountFormData) {
  if (!editingAccount.value) return;
  mutationError.value = null;
  try {
    await client.update(editingAccount.value.id, data);
    showEditDialog.value = false;
    editingAccount.value = null;
    await refreshAccounts();
  } catch (e) {
    mutationError.value = String(e);
  }
}

const showArchiveDialog = ref(false);
const archivingAccount = ref<Account | null>(null);
const archiveCascadeCount = ref(0);

function countDescendants(accountId: string, rows: typeof tree.value): number {
  for (const r of rows) {
    if (r.id === accountId) return countAll(r);
    const found = countDescendants(accountId, r.subRows);
    if (found >= 0) return found;
  }
  return -1;
}

function countAll(row: typeof tree.value[0]): number {
  let count = row.subRows.length;
  for (const c of row.subRows) count += countAll(c);
  return count;
}

function openArchive(account: Account) {
  archivingAccount.value = account;
  archiveCascadeCount.value = Math.max(0, countDescendants(account.id, tree.value));
  showArchiveDialog.value = true;
  mutationError.value = null;
}

async function confirmArchive() {
  if (!archivingAccount.value) return;
  mutationError.value = null;
  try {
    await client.archive(archivingAccount.value.id);
    showArchiveDialog.value = false;
    archivingAccount.value = null;
    await refreshAccounts();
  } catch (e) {
    mutationError.value = String(e);
  }
}

const showDeleteDialog = ref(false);
const deletingAccount = ref<Account | null>(null);
const deleteCascadeCount = ref(0);

function openDelete(account: Account) {
  deletingAccount.value = account;
  deleteCascadeCount.value = Math.max(0, countDescendants(account.id, tree.value));
  showDeleteDialog.value = true;
  mutationError.value = null;
}

async function confirmDelete() {
  if (!deletingAccount.value) return;
  mutationError.value = null;
  try {
    await client.delete(deletingAccount.value.id);
    showDeleteDialog.value = false;
    deletingAccount.value = null;
    await refreshAccounts();
  } catch (e) {
    mutationError.value = String(e);
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex items-center justify-between gap-4">
      <h1 class="text-xl font-bold text-on-background">Accounts</h1>
      <div class="flex items-center gap-3">
        <label class="flex items-center gap-2 text-sm text-on-surface-variant cursor-pointer select-none">
          <input type="checkbox" :checked="showArchived" @change="showArchived = !showArchived" />
          Show archived
        </label>
      </div>
    </div>

    <AppInput v-model="searchQuery" placeholder="Search accounts by name or tag…" />

    <div
      v-if="mutationError"
      class="rounded-lg border border-error bg-error/10 text-error px-4 py-3 text-sm"
    >
      {{ mutationError }}
    </div>

    <div
      v-if="status === 'pending'"
      class="text-center py-16 text-on-surface-variant"
    >
      Loading…
    </div>

    <AccountTable
      v-else-if="tree.length"
      :data="tree"
      @create="openCreate"
      @edit="openEdit"
      @archive="openArchive"
      @delete="openDelete"
    />

    <AppDialog v-model:open="showCreateDialog" size="md">
      <template #title>Add Account</template>
      <AccountForm
        :parent-path="creatingParentPath"
        @submit="handleCreate"
        @cancel="showCreateDialog = false"
      />
    </AppDialog>

    <AppDialog v-model:open="showEditDialog" size="md">
      <template #title>Edit Account</template>
      <AccountForm
        v-if="editingAccount"
        :initial="{
          name: editingAccount.name,
          description: editingAccount.description,
          tags: editingAccount.tags,
        }"
        :parent-path="getParentPath(editingAccount.parent_id ?? '')"
        @submit="handleUpdate"
        @cancel="showEditDialog = false"
      />
    </AppDialog>

    <AppDialog v-model:open="showArchiveDialog" size="sm">
      <template #title>Archive Account</template>
      <AccountArchiveConfirm
        v-if="archivingAccount"
        :account="archivingAccount"
        :cascade-count="archiveCascadeCount"
        @confirm="confirmArchive"
        @cancel="showArchiveDialog = false"
      />
    </AppDialog>

    <AppDialog v-model:open="showDeleteDialog" size="sm">
      <template #title>Delete Account</template>
      <AccountDeleteConfirm
        v-if="deletingAccount"
        :account="deletingAccount"
        :cascade-count="deleteCascadeCount"
        @confirm="confirmDelete"
        @cancel="showDeleteDialog = false"
      />
    </AppDialog>
  </div>
</template>
