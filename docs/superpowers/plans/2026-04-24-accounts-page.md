# Accounts Page + Journal Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the Accounts page (`/journals/:id/accounts`) as an expandable TanStack Table and a minimal Journal Dashboard (`/journals/:id`).

**Architecture:** 5 layers. Layer 1 exposes `archive_account` in Tauri. Layer 2 builds shared frontend foundation (model, client, composables, `journal.vue` layout). Layer 3 enhances `AppDataTable` with TanStack expanding. Layer 4 builds the Accounts page (table + form + dialogs). Layer 5 builds the minimal dashboard. TDD throughout — tests first, then implementation.

**Tech Stack:** Nuxt 4 (Vue 3 + TypeScript), TanStack Table (expanding), Tauri (Rust), Vitest, `@vue/test-utils`

**Spec:** `docs/superpowers/specs/2026-04-24-accounts-page-design.md`
**Product docs:** `docs/product/pages/accounts.md`, `docs/product/pages/journal.md`

---

### Task 1: Expose archive_account Tauri command

**Files:**
- Modify: `packages/endpoint-tauri/endpoint-tauri/src/account.rs`
- Modify: `packages/endpoint-tauri/endpoint-tauri/src/lib.rs`

- [ ] **Step 1: Read existing account.rs for pattern**

Read `packages/endpoint-tauri/endpoint-tauri/src/account.rs` — note how `create_account`, `update_account`, `delete_account` are structured. Each takes `state: tauri::State<'_, AppState>` and uses `state.account_service.handle(...)`.

- [ ] **Step 2: Add archive_account command**

Add after `delete_account` in `packages/endpoint-tauri/endpoint-tauri/src/account.rs`:

```rust
#[tauri::command]
pub async fn archive_account(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<AccountResponse, String> {
    let mut sess = state.new_session();
    let account_id = domain::account::AccountId::from_value(&id);

    let cmd = domain::account::command::AccountCommand::Archive {
        id: account_id,
        archived_at: chrono::Utc::now(),
    };

    let result = state
        .account_service
        .handle(&mut sess, cmd)
        .await
        .map_err(CommandError::from_domain)?;

    let account = result
        .entities
        .into_iter()
        .next()
        .ok_or_else(|| format!("account {id} not found after archive"))?;

    Ok(AccountResponse::from_account(&account))
}
```

- [ ] **Step 3: Verify AccountCommand::Archive variant**

Check `crates/domain/src/account/command.rs` to confirm the exact variant name for `AccountCommand::Archive`. It may be a struct variant with fields `id` and `archived_at`. Read the file and adjust the command construction if needed.

Expected: `AccountCommand::Archive` exists as a variant with `id: AccountId` and `archived_at: DateTime<Utc>` fields, or similar.

- [ ] **Step 4: Register archive_account in lib.rs**

In `packages/endpoint-tauri/endpoint-tauri/src/lib.rs`, add `account::archive_account` to the `generate_handler!` macro:

```rust
pub fn register_handlers<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        journal::create_journal,
        journal::get_journal,
        journal::list_journals,
        journal::update_journal,
        journal::delete_journal,
        account::create_account,
        account::get_account,
        account::list_accounts,
        account::update_account,
        account::delete_account,
        account::archive_account,
    ])
}
```

- [ ] **Step 5: Build and verify**

```bash
cargo build -p endpoint-tauri
```

Expected: compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add packages/endpoint-tauri/endpoint-tauri/src/account.rs packages/endpoint-tauri/endpoint-tauri/src/lib.rs
git commit -m "feat: expose archive_account Tauri command"
```

---

### Task 2: Account model

**Files:**
- Create: `packages/shared/app/models/account.ts`
- Modify: `packages/shared/index.ts`
- Test: `packages/shared/app/models/account.test.ts`

- [ ] **Step 1: Read existing model pattern**

Read `packages/shared/app/models/journal.ts` — note the pattern: interfaces for `Journal`, `CreateJournalRequest`, `UpdateJournalRequest`, `JournalFilter`, `JournalFormData`. The file exports types only (no runtime code).

- [ ] **Step 2: Write the test file**

Create `packages/shared/app/models/account.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { AccountType } from "./account";

describe("AccountType", () => {
  it("has the five root account types", () => {
    expect(AccountType.Asset).toBe("Asset");
    expect(AccountType.Liability).toBe("Liability");
    expect(AccountType.Equity).toBe("Equity");
    expect(AccountType.Income).toBe("Income");
    expect(AccountType.Expense).toBe("Expense");
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

```bash
pnpm vitest run packages/shared/app/models/account.test.ts
```

Expected: FAIL — module not found.

- [ ] **Step 4: Write the model file**

Create `packages/shared/app/models/account.ts`:

```ts
export enum AccountType {
  Asset = "Asset",
  Liability = "Liability",
  Equity = "Equity",
  Income = "Income",
  Expense = "Expense",
}

export interface Account {
  id: string;
  journal_id: string;
  parent_id: string | null;
  type: AccountType;
  name: string;
  description: string;
  tags: string[];
  created_at: string | null;
  last_modified_at: string | null;
  archived_at: string | null;
}

export interface CreateAccountRequest {
  journal_id: string;
  parent_id: string;
  name: string;
  description?: string;
  tags?: string[];
}

export interface UpdateAccountRequest {
  name?: string;
  description?: string;
  tags?: string[];
}

export interface AccountFilter {
  id?: string;
  journal_id?: string;
  parent_id?: string;
  name?: string;
  type?: string;
  tag?: string;
  fullText?: string;
}

export interface AccountFormData {
  name: string;
  description: string;
  tags: string[];
}

export interface AccountRow extends Account {
  subRows: AccountRow[];
  depth: number;
}
```

- [ ] **Step 5: Run test to verify it passes**

```bash
pnpm vitest run packages/shared/app/models/account.test.ts
```

Expected: PASS.

- [ ] **Step 6: Check existing model index**

Read `packages/shared/app/models/index.ts` to see what it exports.

- [ ] **Step 7: Export from models barrel**

Edit `packages/shared/app/models/index.ts` — add `export * from "./account";` (or add account types individually if the file uses explicit exports).

- [ ] **Step 8: Export Account types from shared package**

Read `packages/shared/index.ts`. Add:

```ts
export type {
  Account,
  AccountRow,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountFilter,
  AccountFormData,
} from "./app/models";
export { AccountType } from "./app/models";
```

- [ ] **Step 9: Commit**

```bash
git add packages/shared/app/models/account.ts packages/shared/app/models/account.test.ts packages/shared/app/models/index.ts packages/shared/index.ts
git commit -m "feat: add Account model types"
```

---

### Task 3: AccountClient interface

**Files:**
- Create: `packages/shared/app/clients/account-client.ts`
- Modify: `packages/shared/index.ts`

- [ ] **Step 1: Read existing client pattern**

Read `packages/shared/app/clients/journal-client.ts` — note the `JournalClient` interface with `create/get/list/update/delete`, the `declare module "#app"` block for type augmentation.

- [ ] **Step 2: Write the client interface**

Create `packages/shared/app/clients/account-client.ts`:

```ts
import type {
  Account,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountFilter,
} from "../models";

export interface AccountClient {
  create(request: CreateAccountRequest): Promise<Account>;
  get(id: string): Promise<Account>;
  list(filter?: AccountFilter): Promise<Account[]>;
  update(id: string, request: UpdateAccountRequest): Promise<Account>;
  delete(id: string): Promise<void>;
  archive(id: string): Promise<Account>;
}

declare module "#app" {
  interface NuxtApp {
    $accountClient: AccountClient;
  }
}
```

- [ ] **Step 3: Export AccountClient from shared package**

Read `packages/shared/index.ts`. Add:

```ts
export type { AccountClient } from "./app/clients";
```

- [ ] **Step 4: Check clients barrel**

Read `packages/shared/app/clients/index.ts`. Add `export type { AccountClient } from "./account-client";` if using explicit exports.

- [ ] **Step 5: Commit**

```bash
git add packages/shared/app/clients/account-client.ts packages/shared/app/clients/index.ts packages/shared/index.ts
git commit -m "feat: add AccountClient interface"
```

---

### Task 4: TauriAccountClient implementation + plugin

**Files:**
- Create: `packages/endpoint-tauri/app/clients/account-client.ts`
- Create: `packages/endpoint-tauri/app/plugins/account-client.ts`

- [ ] **Step 1: Read existing Tauri client pattern**

Read `packages/endpoint-tauri/app/clients/journal-client.ts` — note the `invoke(...)` calls mapping to Tauri commands.

- [ ] **Step 2: Write TauriAccountClient**

Create `packages/endpoint-tauri/app/clients/account-client.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  AccountClient,
  Account,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountFilter,
} from "@white-rabbit/shared";

export class TauriAccountClient implements AccountClient {
  async create(request: CreateAccountRequest): Promise<Account> {
    return invoke("create_account", { request });
  }

  async get(id: string): Promise<Account> {
    return invoke("get_account", { id });
  }

  async list(filter?: AccountFilter): Promise<Account[]> {
    return invoke("list_accounts", { filter: filter ?? {} });
  }

  async update(id: string, request: UpdateAccountRequest): Promise<Account> {
    return invoke("update_account", { id, request });
  }

  async delete(id: string): Promise<void> {
    return invoke("delete_account", { id });
  }

  async archive(id: string): Promise<Account> {
    return invoke("archive_account", { id });
  }
}
```

- [ ] **Step 3: Read existing plugin pattern**

Read `packages/endpoint-tauri/app/plugins/journal-client.ts` — note the `defineNuxtPlugin` pattern.

- [ ] **Step 4: Write account client plugin**

Create `packages/endpoint-tauri/app/plugins/account-client.ts`:

```ts
import { TauriAccountClient } from "~/clients";

export default defineNuxtPlugin(() => ({
  provide: { accountClient: new TauriAccountClient() },
}));
```

- [ ] **Step 5: Commit**

```bash
git add packages/endpoint-tauri/app/clients/account-client.ts packages/endpoint-tauri/app/plugins/account-client.ts
git commit -m "feat: add TauriAccountClient + plugin"
```

---

### Task 5: useAccountClient composable

**Files:**
- Create: `packages/shared/app/composables/useAccountClient.ts`

- [ ] **Step 1: Read existing composable pattern**

Read `packages/shared/app/composables/useJournalClient.ts` — note the simple `useNuxtApp().$journalClient` pattern.

- [ ] **Step 2: Write useAccountClient**

Create `packages/shared/app/composables/useAccountClient.ts`:

```ts
import type { AccountClient } from "../clients";

export function useAccountClient(): AccountClient {
  return useNuxtApp().$accountClient as AccountClient;
}
```

- [ ] **Step 3: Commit**

```bash
git add packages/shared/app/composables/useAccountClient.ts
git commit -m "feat: add useAccountClient composable"
```

---

### Task 6: Account composables (useAccounts, useAccount, useAccountTree)

**Files:**
- Create: `packages/shared/app/composables/account.ts`
- Test: `packages/shared/app/composables/account.test.ts`

- [ ] **Step 1: Read existing composable pattern**

Read `packages/shared/app/composables/journal.ts` — note the `useAsyncData` pattern with `watch`.

- [ ] **Step 2: Write the test for useAccountTree**

Create `packages/shared/app/composables/account.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { buildAccountRows } from "./account";
import type { Account } from "../models";
import { AccountType } from "../models";

function makeAcc(overrides: Partial<Account> & { id: string }): Account {
  return {
    journal_id: "j1",
    parent_id: null,
    type: AccountType.Asset,
    name: "Test",
    description: "",
    tags: [],
    created_at: null,
    last_modified_at: null,
    archived_at: null,
    ...overrides,
  };
}

describe("buildAccountRows", () => {
  it("returns 5 root rows from the 5 root accounts", () => {
    const accounts: Account[] = [
      makeAcc({ id: "a1", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "l1", type: AccountType.Liability, parent_id: null, name: "Liability" }),
      makeAcc({ id: "e1", type: AccountType.Equity, parent_id: null, name: "Equity" }),
      makeAcc({ id: "i1", type: AccountType.Income, parent_id: null, name: "Income" }),
      makeAcc({ id: "x1", type: AccountType.Expense, parent_id: null, name: "Expense" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows).toHaveLength(5);
    expect(rows.map(r => r.type)).toEqual([
      AccountType.Asset,
      AccountType.Liability,
      AccountType.Equity,
      AccountType.Income,
      AccountType.Expense,
    ]);
  });

  it("nests children under parents via subRows", () => {
    const accounts: Account[] = [
      makeAcc({ id: "root", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "child1", type: AccountType.Asset, parent_id: "root", name: "Bank" }),
      makeAcc({ id: "child2", type: AccountType.Asset, parent_id: "root", name: "Cash" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows).toHaveLength(1);
    expect(rows[0].subRows).toHaveLength(2);
    expect(rows[0].subRows[0].name).toBe("Bank");
    expect(rows[0].subRows[1].name).toBe("Cash");
  });

  it("computes depth correctly", () => {
    const accounts: Account[] = [
      makeAcc({ id: "root", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "c1", type: AccountType.Asset, parent_id: "root", name: "Bank" }),
      makeAcc({ id: "c2", type: AccountType.Asset, parent_id: "c1", name: "Checking" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows[0].depth).toBe(0);
    expect(rows[0].subRows[0].depth).toBe(1);
    expect(rows[0].subRows[0].subRows[0].depth).toBe(2);
  });

  it("handles empty account list", () => {
    const rows = buildAccountRows([]);
    expect(rows).toHaveLength(0);
  });

  it("groups accounts by type", () => {
    const accounts: Account[] = [
      makeAcc({ id: "a1", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "e1", type: AccountType.Expense, parent_id: null, name: "Expense" }),
      makeAcc({ id: "ea1", type: AccountType.Asset, parent_id: "a1", name: "Cash" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows).toHaveLength(2);
    const assetRow = rows.find(r => r.type === AccountType.Asset)!;
    expect(assetRow.subRows).toHaveLength(1);
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

```bash
pnpm vitest run packages/shared/app/composables/account.test.ts
```

Expected: FAIL — `buildAccountRows` not exported.

- [ ] **Step 4: Write the composable**

Create `packages/shared/app/composables/account.ts`:

```ts
import type { Account, AccountRow, AccountFilter } from "../models";
import { AccountType } from "../models";

export function useAccounts(journalId: MaybeRef<string>) {
  const client = useAccountClient();
  return useAsyncData(`accounts:${toValue(journalId)}`, () =>
    client.list({ journal_id: toValue(journalId) }),
    { watch: isRef(journalId) ? [journalId] : undefined }
  );
}

export function useAccount(id: MaybeRef<string>) {
  const client = useAccountClient();
  return useAsyncData(`account:${toValue(id)}`, () =>
    client.get(toValue(id)),
    { watch: isRef(id) ? [id] : undefined }
  );
}

export function buildAccountRows(accounts: Account[]): AccountRow[] {
  const byType = new Map<AccountType, Account[]>();
  const byParentId = new Map<string | null, Account[]>();
  const byId = new Map<string, AccountRow>();

  for (const a of accounts) {
    if (!byType.has(a.type)) byType.set(a.type, []);
    byType.get(a.type)!.push(a);
  }

  for (const a of accounts) {
    const key = a.parent_id ?? null;
    if (!byParentId.has(key)) byParentId.set(key, []);
    byParentId.get(key)!.push(a);
  }

  function buildRow(account: Account, depth: number): AccountRow {
    const row: AccountRow = { ...account, subRows: [], depth };
    byId.set(account.id, row);
    const children = byParentId.get(account.id) ?? [];
    row.subRows = children.map(c => buildRow(c, depth + 1));
    return row;
  }

  const roots: AccountRow[] = [];
  for (const type of [AccountType.Asset, AccountType.Liability, AccountType.Equity, AccountType.Income, AccountType.Expense]) {
    const typeRoots = byParentId.get(null)?.filter(a => a.type === type) ?? [];
    for (const a of typeRoots) {
      roots.push(buildRow(a, 0));
    }
  }

  return roots;
}

export function useAccountTree(accounts: Ref<Account[]>, showArchived: Ref<boolean>) {
  const tree = computed<AccountRow[]>(() => {
    const filtered = showArchived.value
      ? accounts.value
      : accounts.value.filter(a => !a.archived_at);
    return buildAccountRows(filtered);
  });
  return { tree };
}
```

- [ ] **Step 5: Run test to verify it passes**

```bash
pnpm vitest run packages/shared/app/composables/account.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/shared/app/composables/account.ts packages/shared/app/composables/account.test.ts
git commit -m "feat: add account composables (useAccounts, useAccount, useAccountTree)"
```

---

### Task 7: useCurrentJournal composable

**Files:**
- Create: `packages/shared/app/composables/useCurrentJournal.ts`

- [ ] **Step 1: Write useCurrentJournal**

Create `packages/shared/app/composables/useCurrentJournal.ts`:

```ts
export function useCurrentJournal() {
  const route = useRoute();
  const journalId = computed(() => route.params.id as string);
  const { data: journal, refresh, status } = useJournal(journalId);
  return { journal, journalId, refresh, status };
}
```

This composable assumes the route param is `id` (from `/journals/:id/...`). It uses the existing `useJournal` composable.

- [ ] **Step 2: Commit**

```bash
git add packages/shared/app/composables/useCurrentJournal.ts
git commit -m "feat: add useCurrentJournal composable"
```

---

### Task 8: layouts/journal.vue

**Files:**
- Create: `packages/shared/app/layouts/journal.vue`

- [ ] **Step 1: Read existing layout pattern**

Read `packages/shared/app/layouts/default.vue` — note the `<header>`, `<main>`, `<footer>` structure and the `<slot />`.

- [ ] **Step 2: Write journal.vue layout**

Create `packages/shared/app/layouts/journal.vue`:

```vue
<script setup lang="ts">
const route = useRoute();
const journalId = computed(() => route.params.id as string);
const { data: journal } = useJournal(journalId);

const menuItems = [
  { label: "Dashboard", to: `/journals/${journalId.value}`, enabled: true },
  { label: "Accounts", to: `/journals/${journalId.value}/accounts`, enabled: true },
  { label: "Records", to: "", enabled: false },
  { label: "Reports", to: "", enabled: false },
];

function isActive(path: string) {
  return route.path === path;
}
</script>

<template>
  <NuxtLayout name="default">
    <div class="flex">
      <aside class="w-48 shrink-0 border-r border-outline-variant p-3">
        <nav class="flex flex-col gap-1">
          <template v-for="item in menuItems" :key="item.label">
            <NuxtLink
              v-if="item.enabled"
              :to="item.to"
              class="block px-3 py-2 rounded text-sm transition-colors"
              :class="isActive(item.to)
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-on-surface-variant hover:bg-surface-variant'"
            >
              {{ item.label }}
            </NuxtLink>
            <span
              v-else
              class="block px-3 py-2 rounded text-sm text-on-surface-variant/30 cursor-not-allowed select-none"
            >
              {{ item.label }}
            </span>
          </template>
        </nav>
      </aside>
      <main class="flex-1 p-4 min-w-0">
        <slot />
      </main>
    </div>
  </NuxtLayout>
</template>
```

Note: `menuItems` uses `to` for active links and `enabled: false` for disabled placeholder items (Records, Reports). The `NuxtLink` used for active items; `<span>` for disabled placeholders.

`journalId` is a computed ref — using it in template `:to` expressions works because Vue unwraps refs in templates.

- [ ] **Step 3: Commit**

```bash
git add packages/shared/app/layouts/journal.vue
git commit -m "feat: add journal-scoped layout with side menu"
```

---

### Task 9: Enhance AppDataTable with expanding support

**Files:**
- Modify: `packages/shared/app/components/ui/AppDataTable.vue`

- [ ] **Step 1: Read current AppDataTable**

Read `packages/shared/app/components/ui/AppDataTable.vue` — note the existing props (`data`, `columns`), the `useVueTable` call with `getCoreRowModel`, `getSortedRowModel`, `getFilteredRowModel`. Note the template rendering `table.getRowModel().rows`.

- [ ] **Step 2: Read existing AppDataTable test**

Read `packages/shared/app/components/ui/AppDataTable.vue` based tests — there may be `AppDataTable.test.ts` or similar. We need to ensure existing tests pass after the change.

- [ ] **Step 3: Add expanding support to AppDataTable**

Modify `packages/shared/app/components/ui/AppDataTable.vue`. The changes are:

**Imports — add:**
```ts
import { getExpandedRowModel } from "@tanstack/vue-table";
import type { ExpandedState } from "@tanstack/vue-table";
import { ref as importedRef } from "vue"; // if not already imported
```

**Props — add `getSubRows` and `enableExpanding`:**
```ts
const props = defineProps<{
  data: TData[];
  columns: ColumnDef<TData, unknown>[];
  getSubRows?: (row: TData) => TData[];
  enableExpanding?: boolean;
}>();
```

**State — add expanded state:**
```ts
const expanded = importedRef<ExpandedState>({});
```

**Table options — add expanding options conditionally:**
In the `useVueTable` call, add these only when `enableExpanding` is true:

```ts
const table = useVueTable({
  get data() { return props.data; },
  get columns() { return props.columns; },
  state: {
    get sorting() { return sorting.value; },
    get columnFilters() { return columnFilters.value; },
    ...(props.enableExpanding ? { get expanded() { return expanded.value; } } : {}),
  },
  onSortingChange: (updater) => {
    sorting.value = typeof updater === "function" ? updater(sorting.value) : updater;
  },
  onColumnFiltersChange: (updater) => {
    columnFilters.value = typeof updater === "function" ? updater(columnFilters.value) : updater;
  },
  onExpandedChange: props.enableExpanding
    ? (updater: any) => {
        expanded.value = typeof updater === "function" ? updater(expanded.value) : updater;
      }
    : undefined,
  getSubRows: props.getSubRows,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getExpandedRowModel: props.enableExpanding ? getExpandedRowModel() : undefined,
});
```

- [ ] **Step 4: Run existing tests to verify no regression**

```bash
pnpm vitest run packages/shared/app/components/ui/
```

Expected: All existing UI component tests PASS. AppDataTable without expanding props works identically.

- [ ] **Step 5: Commit**

```bash
git add packages/shared/app/components/ui/AppDataTable.vue
git commit -m "feat: add optional expanding support to AppDataTable"
```

---

### Task 10: AccountTable component

**Files:**
- Create: `packages/shared/app/components/AccountTable.vue`
- Test: `packages/shared/app/components/AccountTable.test.ts`

- [ ] **Step 1: Write the test**

Create `packages/shared/app/components/AccountTable.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AccountTable from "./AccountTable.vue";
import { AccountType } from "../models";
import type { AccountRow } from "../models";
import { mountWithPlugins } from "../test-utils-mount";

function makeRow(overrides: Partial<AccountRow> & { id: string }): AccountRow {
  return {
    journal_id: "j1",
    parent_id: null,
    type: AccountType.Asset,
    name: "Asset",
    description: "",
    tags: [],
    created_at: null,
    last_modified_at: null,
    archived_at: null,
    subRows: [],
    depth: 0,
    ...overrides,
  };
}

describe("AccountTable", () => {
  it("renders the 5 root accounts", () => {
    const data: AccountRow[] = [
      makeRow({ id: "a1", type: AccountType.Asset, name: "Asset" }),
      makeRow({ id: "l1", type: AccountType.Liability, name: "Liability" }),
      makeRow({ id: "e1", type: AccountType.Equity, name: "Equity" }),
      makeRow({ id: "i1", type: AccountType.Income, name: "Income" }),
      makeRow({ id: "x1", type: AccountType.Expense, name: "Expense" }),
    ];
    const wrapper = mountWithPlugins(AccountTable, { props: { data } });
    expect(wrapper.text()).toContain("Asset");
    expect(wrapper.text()).toContain("Liability");
    expect(wrapper.text()).toContain("Equity");
    expect(wrapper.text()).toContain("Income");
    expect(wrapper.text()).toContain("Expense");
  });

  it('emits "create" when add-child button is clicked', async () => {
    const data: AccountRow[] = [
      makeRow({ id: "a1", type: AccountType.Asset, name: "Asset" }),
    ];
    const wrapper = mountWithPlugins(AccountTable, { props: { data } });
    // Find any button with lucide:plus icon in the actions column
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const plusBtn = buttons.find(b => b.attributes("aria-label")?.includes("Add"));
    if (plusBtn) {
      await plusBtn.trigger("click");
      expect(wrapper.emitted("create")).toBeTruthy();
      expect(wrapper.emitted("create")![0]).toEqual(["a1"]);
    }
  });

  it("shows archived badge for archived accounts", () => {
    const data: AccountRow[] = [
      makeRow({ id: "c1", type: AccountType.Asset, name: "Bank", parent_id: "a1", parentId: "a1", archived_at: "2024-01-01" }),
    ];
    const wrapper = mountWithPlugins(AccountTable, { props: { data } });
    expect(wrapper.text()).toContain("Archived");
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

```bash
pnpm vitest run packages/shared/app/components/AccountTable.test.ts
```

Expected: FAIL — component not found.

- [ ] **Step 4: Read test-utils-mount for pattern**

Read `packages/shared/app/test-utils-mount.ts` to understand `mountWithPlugins`.

- [ ] **Step 5: Write AccountTable component**

Create `packages/shared/app/components/AccountTable.vue`:

```vue
<script setup lang="ts">
import { h } from "vue";
import type { AccountRow, Account } from "../models";
import AppDataTable from "./ui/AppDataTable.vue";
import AppButton from "./ui/AppButton.vue";
import AppIcon from "./ui/AppIcon.vue";
import AppChip from "./ui/AppChip.vue";
import type { ColumnDef } from "@tanstack/vue-table";

const props = defineProps<{
  data: AccountRow[];
}>();

const emit = defineEmits<{
  create: [parentId: string];
  edit: [account: Account];
  archive: [account: Account];
  delete: [account: Account];
}>();

const TYPE_ICONS: Record<string, string> = {
  Asset: "lucide:landmark",
  Liability: "lucide:credit-card",
  Equity: "lucide:scale",
  Income: "lucide:trending-up",
  Expense: "lucide:trending-down",
};

const columns: ColumnDef<AccountRow, unknown>[] = [
  {
    id: "name",
    header: "Name",
    cell: ({ row }) => {
      const account = row.original;
      return h("div", { class: "flex items-center gap-2" }, [
        row.getCanExpand()
          ? h(AppButton, {
              variant: "ghost",
              size: "sm",
              "aria-label": account.name,
              onClick: row.getToggleExpandedHandler(),
            }, () => h(AppIcon, {
              icon: row.getIsExpanded() ? "lucide:chevron-down" : "lucide:chevron-right",
              size: "sm",
            }))
          : h("span", { class: "w-6 inline-block" }),

        h(AppIcon, {
          icon: TYPE_ICONS[account.type] ?? "lucide:folder",
          size: "sm",
        }),

        h("span", {
          class: account.parent_id === null ? "font-semibold text-on-surface" : "text-on-surface",
          style: { paddingLeft: `${account.depth * 12}px` },
        }, account.name),

        account.archived_at
          ? h("span", { class: "text-xs px-1.5 py-0.5 rounded bg-surface-variant text-on-surface-variant ml-2" }, "Archived")
          : null,
      ]);
    },
  },
  {
    id: "tags",
    header: "Tags",
    cell: ({ row }) => {
      const tags = row.original.tags;
      if (!tags.length) return null;
      return h("div", { class: "flex flex-wrap gap-1" },
        tags.map(tag => h(AppChip, { variant: "tonal", size: "sm" }, () => tag))
      );
    },
  },
  {
    id: "actions",
    header: "",
    cell: ({ row }) => {
      const account = row.original;
      const isRoot = account.parent_id === null;
      const isArchived = !!account.archived_at;

      const buttons: ReturnType<typeof h>[] = [];

      buttons.push(
        h(AppButton, {
          variant: "ghost", size: "sm",
          "aria-label": `Add child account under ${account.name}`,
          onClick: () => emit("create", account.id),
        }, () => h(AppIcon, { icon: "lucide:plus", size: "sm" }))
      );

      if (!isRoot) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            "aria-label": `Edit ${account.name}`,
            onClick: () => emit("edit", account as Account),
          }, () => h(AppIcon, { icon: "lucide:pencil", size: "sm" }))
        );
      }

      if (!isRoot && !isArchived) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            "aria-label": `Archive ${account.name}`,
            onClick: () => emit("archive", account as Account),
          }, () => h(AppIcon, { icon: "lucide:archive", size: "sm" }))
        );
      }

      if (!isRoot) {
        buttons.push(
          h(AppButton, {
            variant: "ghost", size: "sm",
            class: "text-error",
            "aria-label": `Delete ${account.name}`,
            onClick: () => emit("delete", account as Account),
          }, () => h(AppIcon, { icon: "lucide:trash-2", size: "sm" }))
        );
      }

      return h("div", { class: "flex items-center gap-1" }, buttons);
    },
  },
];

function getSubRows(row: AccountRow): AccountRow[] {
  return row.subRows;
}
</script>

<template>
  <AppDataTable
    :data="data"
    :columns="columns"
    :get-sub-rows="getSubRows"
    :enable-expanding="true"
  >
    <template #empty>
      <div class="text-center py-8 text-on-surface-variant">
        No accounts found. Create one with the [+] button on a root account.
      </div>
    </template>
  </AppDataTable>
</template>
```

- [ ] **Step 6: Run test to verify it passes**

```bash
pnpm vitest run packages/shared/app/components/AccountTable.test.ts
```

Expected: PASS (may need adjustment for `mountWithPlugins` pattern — read the helper first).

- [ ] **Step 7: Commit**

```bash
git add packages/shared/app/components/AccountTable.vue packages/shared/app/components/AccountTable.test.ts
git commit -m "feat: add AccountTable component with expanding support"
```

---

### Task 11: AccountForm component

**Files:**
- Create: `packages/shared/app/components/AccountForm.vue`
- Test: `packages/shared/app/components/AccountForm.test.ts`

- [ ] **Step 1: Read existing form pattern**

Read `packages/shared/app/components/JournalForm.vue` — note the `initial` prop pattern, `watch` for prop changes, `emit("submit", data)` pattern. Mirror this for `AccountForm`.

- [ ] **Step 2: Write the test**

Create `packages/shared/app/components/AccountForm.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AccountForm from "./AccountForm.vue";
import { mountWithPlugins } from "../test-utils-mount";

describe("AccountForm", () => {
  it("renders name and description fields", () => {
    const wrapper = mountWithPlugins(AccountForm, {
      props: { parentPath: "Asset > Bank" },
    });
    expect(wrapper.text()).toContain("Asset > Bank");
  });

  it('emits submit with form data', async () => {
    const wrapper = mountWithPlugins(AccountForm, {
      props: { parentPath: "" },
    });

    const nameInput = wrapper.find("#account-name");
    await nameInput.setValue("Checking");

    const form = wrapper.find("form");
    await form.trigger("submit.prevent");

    expect(wrapper.emitted("submit")).toBeTruthy();
    expect(wrapper.emitted("submit")![0][0]).toMatchObject({
      name: "Checking",
      description: "",
      tags: [],
    });
  });

  it('emits cancel when cancel button is clicked', async () => {
    const wrapper = mountWithPlugins(AccountForm, {
      props: { parentPath: "" },
    });
    const cancelBtn = wrapper.findAllComponents({ name: "AppButton" })
      .find(b => b.text() === "Cancel");
    if (cancelBtn) {
      await cancelBtn.trigger("click");
      expect(wrapper.emitted("cancel")).toBeTruthy();
    }
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

```bash
pnpm vitest run packages/shared/app/components/AccountForm.test.ts
```

Expected: FAIL — component not found.

- [ ] **Step 4: Write AccountForm component**

Create `packages/shared/app/components/AccountForm.vue`:

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
import type { AccountFormData } from "../models";
import AppInput from "./ui/AppInput.vue";
import AppTextarea from "./ui/AppTextarea.vue";
import AppButton from "./ui/AppButton.vue";
import AppTagInput from "./ui/AppTagInput.vue";

const props = defineProps<{
  initial?: { name: string; description: string; tags: string[] };
  parentPath?: string;
}>();

const emit = defineEmits<{
  submit: [data: AccountFormData];
  cancel: [];
}>();

const RESERVED_NAMES = ["Asset", "Liability", "Equity", "Income", "Expense"];

const name = ref(props.initial?.name ?? "");
const description = ref(props.initial?.description ?? "");
const tags = ref<string[]>(props.initial?.tags ?? []);
const error = ref<string | null>(null);

watch(
  () => props.initial,
  (val) => {
    name.value = val?.name ?? "";
    description.value = val?.description ?? "";
    tags.value = val?.tags ?? [];
  },
);

function validate(): boolean {
  if (!name.value.trim()) {
    error.value = "Name is required.";
    return false;
  }
  if (RESERVED_NAMES.some(r => r.localeCompare(name.value, undefined, { sensitivity: "base" }) === 0)) {
    error.value = `"${name.value}" is a reserved root account name.`;
    return false;
  }
  error.value = null;
  return true;
}

function handleSubmit() {
  if (!validate()) return;
  emit("submit", {
    name: name.value,
    description: description.value,
    tags: tags.value,
  });
}
</script>

<template>
  <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
    <div v-if="parentPath" class="text-sm text-on-surface-variant">
      Parent: <span class="font-medium">{{ parentPath }}</span>
    </div>

    <div>
      <label for="account-name" class="block text-sm font-medium mb-1">Name</label>
      <AppInput
        id="account-name"
        v-model="name"
        placeholder="Account name"
        required
      />
    </div>
    <div>
      <label for="account-description" class="block text-sm font-medium mb-1">Description</label>
      <AppTextarea
        id="account-description"
        v-model="description"
        placeholder="Optional description"
        :rows="3"
      />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Tags</label>
      <AppTagInput v-model="tags" placeholder="Add tag…" />
    </div>

    <div v-if="error" class="text-error text-sm">{{ error }}</div>

    <div class="flex justify-end gap-2 pt-2">
      <AppButton type="button" variant="outlined" @click="$emit('cancel')">
        Cancel
      </AppButton>
      <AppButton type="submit" variant="solid">
        {{ initial ? "Update" : "Create" }}
      </AppButton>
    </div>
  </form>
</template>
```

- [ ] **Step 5: Run test to verify it passes**

```bash
pnpm vitest run packages/shared/app/components/AccountForm.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/shared/app/components/AccountForm.vue packages/shared/app/components/AccountForm.test.ts
git commit -m "feat: add AccountForm component"
```

---

### Task 12: AccountArchiveConfirm and AccountDeleteConfirm components

**Files:**
- Create: `packages/shared/app/components/AccountArchiveConfirm.vue`
- Create: `packages/shared/app/components/AccountDeleteConfirm.vue`

- [ ] **Step 1: Write AccountArchiveConfirm**

Create `packages/shared/app/components/AccountArchiveConfirm.vue`:

```vue
<script setup lang="ts">
import type { Account } from "../models";
import AppButton from "./ui/AppButton.vue";

defineProps<{
  account: Account;
  cascadeCount: number;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <div class="flex flex-col gap-4">
    <p class="text-sm text-on-surface">
      Archive account <strong>{{ account.name }}</strong>?
    </p>
    <p class="text-sm text-on-surface-variant">
      This will also archive
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}.
      Archived accounts cannot receive new records, but their history is preserved.
    </p>
    <div class="flex justify-end gap-2 pt-2">
      <AppButton variant="outlined" @click="emit('cancel')">Cancel</AppButton>
      <AppButton variant="solid" @click="emit('confirm')">Archive</AppButton>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Write AccountDeleteConfirm**

Create `packages/shared/app/components/AccountDeleteConfirm.vue`:

```vue
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
      Delete account <strong>{{ account.name }}</strong>?
    </p>
    <p class="text-sm text-on-surface-variant">
      This permanently deletes <strong>{{ account.name }}</strong> and
      <strong>{{ cascadeCount }}</strong> descendant
      {{ cascadeCount === 1 ? "account" : "accounts" }}.
      Any records still posting to these accounts will fail to load.
      This action cannot be undone.
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
```

- [ ] **Step 3: Commit**

```bash
git add packages/shared/app/components/AccountArchiveConfirm.vue packages/shared/app/components/AccountDeleteConfirm.vue
git commit -m "feat: add AccountArchiveConfirm and AccountDeleteConfirm components"
```

---

### Task 13: Accounts page (`/journals/:id/accounts`)

**Files:**
- Create: `packages/shared/app/pages/journals/[id]/accounts.vue`

- [ ] **Step 1: Read existing page pattern**

Read `packages/shared/app/pages/index.vue` — note how dialogs are managed, error handling, `useAsyncData` status checks.

- [ ] **Step 2: Write the accounts page**

Create `packages/shared/app/pages/journals/[id]/accounts.vue`:

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import type { Account, AccountFormData } from "../../models";
import AppButton from "../../components/ui/AppButton.vue";
import AppIcon from "../../components/ui/AppIcon.vue";
import AppInput from "../../components/ui/AppInput.vue";
import AppDialog from "../../components/ui/AppDialog.vue";
import AccountTable from "../../components/AccountTable.vue";
import AccountForm from "../../components/AccountForm.vue";
import AccountArchiveConfirm from "../../components/AccountArchiveConfirm.vue";
import AccountDeleteConfirm from "../../components/AccountDeleteConfirm.vue";

definePageMeta({ layout: "journal" });

const { journalId } = useCurrentJournal();
const { data: accounts, status, refresh: refreshAccounts } = useAccounts(journalId);

const showArchived = ref(false);

const { tree } = useAccountTree(
  computed(() => accounts.value ?? []),
  showArchived,
);

const client = useAccountClient();
const mutationError = ref<string | null>(null);

// ── Search ──
const searchQuery = ref("");

// ── Create dialog ──
const showCreateDialog = ref(false);
const creatingParentId = ref<string | null>(null);
const creatingParentPath = ref("");

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

function openCreate(parentId: string) {
  creatingParentId.value = parentId;
  creatingParentPath.value = getParentPath(parentId);
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
      ...data,
    });
    showCreateDialog.value = false;
    await refreshAccounts();
  } catch (e) {
    mutationError.value = String(e);
  }
}

// ── Edit dialog ──
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

// ── Archive dialog ──
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

// ── Delete dialog ──
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
    <!-- Page header -->
    <div class="flex items-center justify-between gap-4">
      <h1 class="text-xl font-bold text-on-background">Accounts</h1>
      <div class="flex items-center gap-3">
        <label class="flex items-center gap-2 text-sm text-on-surface-variant cursor-pointer select-none">
          <input type="checkbox" :checked="showArchived" @change="showArchived = !showArchived" />
          Show archived
        </label>
      </div>
    </div>

    <!-- Search -->
    <AppInput v-model="searchQuery" placeholder="Search accounts by name or tag…" />

    <!-- Error banner -->
    <div
      v-if="mutationError"
      class="rounded-lg border border-error bg-error/10 text-error px-4 py-3 text-sm"
    >
      {{ mutationError }}
    </div>

    <!-- Loading -->
    <div
      v-if="status === 'pending'"
      class="text-center py-16 text-on-surface-variant"
    >
      Loading…
    </div>

    <!-- Table -->
    <AccountTable
      v-else-if="tree.length"
      :data="tree"
      @create="openCreate"
      @edit="openEdit"
      @archive="openArchive"
      @delete="openDelete"
    />

    <!-- Create dialog -->
    <AppDialog v-model:open="showCreateDialog" size="md">
      <template #title>Add Account</template>
      <AccountForm
        :parent-path="creatingParentPath"
        @submit="handleCreate"
        @cancel="showCreateDialog = false"
      />
    </AppDialog>

    <!-- Edit dialog -->
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

    <!-- Archive dialog -->
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

    <!-- Delete dialog -->
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
```

- [ ] **Step 3: Commit**

```bash
git add packages/shared/app/pages/journals/
git commit -m "feat: add Accounts page with full CRUD"
```

---

### Task 14: Minimal Journal Dashboard (`/journals/:id`)

**Files:**
- Create: `packages/shared/app/pages/journals/[id].vue`

- [ ] **Step 1: Write the dashboard page**

Create `packages/shared/app/pages/journals/[id].vue`:

```vue
<script setup lang="ts">
import AppCard from "../../components/ui/AppCard.vue";
import AppChip from "../../components/ui/AppChip.vue";
import AppIcon from "../../components/ui/AppIcon.vue";

definePageMeta({ layout: "journal" });

const { journal, journalId } = useCurrentJournal();
const { data: accounts } = useAccounts(journalId);

const accountCount = computed(() => accounts.value?.length ?? 0);
</script>

<template>
  <div v-if="journal" class="flex flex-col gap-6">
    <!-- Journal header -->
    <div>
      <h1 class="text-xl font-bold text-on-background">{{ journal.name }}</h1>
      <p v-if="journal.description" class="text-sm text-on-surface-variant mt-1">
        {{ journal.description }}
      </p>
      <div v-if="journal.tags?.length" class="flex flex-wrap gap-1 mt-2">
        <AppChip v-for="tag in journal.tags" :key="tag" variant="tonal" size="sm">
          {{ tag }}
        </AppChip>
      </div>
    </div>

    <!-- Nav cards -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <AppCard
        variant="outlined"
        class="p-4 cursor-pointer hover:border-primary transition-colors"
        @click="navigateTo(`/journals/${journalId}/accounts`)"
      >
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:folder-tree" size="md" class="text-primary shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Accounts</h3>
            <p class="text-sm text-on-surface-variant">
              {{ accountCount }} total
            </p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:list" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Records</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:bar-chart-3" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Reports</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>

      <AppCard variant="outlined" class="p-4 opacity-40 select-none">
        <div class="flex items-center gap-3">
          <AppIcon icon="lucide:settings" size="md" class="text-on-surface-variant/40 shrink-0" />
          <div>
            <h3 class="font-semibold text-on-surface">Settings</h3>
            <p class="text-sm text-on-surface-variant">Coming soon</p>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add packages/shared/app/pages/journals/[id].vue
git commit -m "feat: add minimal Journal Dashboard page"
```

---

### Task 15: Verify JournalCard navigation

**Files:**
- Verify: `packages/shared/app/pages/index.vue`

- [ ] **Step 1: Check JournalCard click handler**

Read `packages/shared/app/pages/index.vue` around the `JournalCard` usage. It should already navigate to `/journals/${journal.id}`.

- [ ] **Step 2: Verify — no changes needed if navigation exists**

The existing code at line 172 (`@click="navigateTo(`/journals/${journal.id}`)"`) already navigates correctly. No changes required.

- [ ] **Step 3: Mark as complete**

No code changes in this task.

---

### Task 16: TypeScript type check and integration test

**Files:**
- All new files

- [ ] **Step 1: Run TypeScript type check**

```bash
npx nuxi typecheck
```

If using pnpm:
```bash
pnpm run typecheck
```

If no script exists, check `package.json` for available scripts.

Expected: No type errors.

- [ ] **Step 2: Fix any type errors**

- Import paths may need adjustment. `AccountTable.vue` imports from `../models` — verify the relative path is correct.
- `useCurrentJournal` imports `useJournal` from `./journal` — should be auto-imported by Nuxt.
- Page components should use `definePageMeta({ layout: "journal" })` or `<NuxtLayout name="journal">`.

- [ ] **Step 3: Run all existing tests**

```bash
pnpm vitest run
```

Expected: All existing tests pass. New tests pass.

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: type errors and test adjustments"
```
