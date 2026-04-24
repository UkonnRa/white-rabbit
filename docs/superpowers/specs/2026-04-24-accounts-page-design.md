# Accounts Page + Journal Dashboard Slice

Implementation plan for shipping the Accounts page (`/journals/:id/accounts`)
and a minimal Journal Dashboard (`/journals/:id`) as a single development slice.

Product specs: `docs/product/pages/accounts.md`, `docs/product/pages/journal.md`.
Feature spec: `docs/product/features.md`.

## 1. Scope

| Included | Deferred |
|----------|----------|
| Expose `archive_account` Tauri command | Records (no Tauri commands, no client, no UI) |
| Account model + client interface + Tauri impl | Reports (no domain support) |
| Account composables (`useAccounts`, `useAccountTree`) | Record form / import dialog |
| `layouts/journal.vue` with side menu | Full three-panel dashboard |
| `useCurrentJournal` composable | Breadcrumb in app header |
| Enhance `AppDataTable` with expanding support | Un-archive (no domain support) |
| Accounts page (table with expandable tree rows) | Commodity/Records count columns (no data yet) |
| AccountForm, ArchiveConfirm, DeleteConfirm dialogs | |
| Minimal dashboard (stats + nav cards) | |
| Deletion guard: disable delete when records reference account | |

## 2. Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Account view | **TanStack Table with expanding, not a custom tree** | Table gives us columns (Tags, future Commodities/Records counts), sorting, filtering for free. Expanding (`getSubRows` + `getExpandedRowModel`) handles hierarchy. See [TanStack Table expanding guide](https://tanstack.com/table/latest/docs/guide/expanding). |
| Table data shape | Flat `Account[]` → nested via `getSubRows: row => row.subRows` | Match TanStack's API. Build the nested structure client-side, same as `accounts.md` §10. |
| Table component | Enhance existing `AppDataTable.vue` with expanding support | Reuse our recipe-driven table. Add `getSubRows` prop, `enableExpanding` prop, expanded state management, `getExpandedRowModel`. |
| `useCurrentJournal` | Route params (no inject/provide) | Simpler. Available everywhere under `journal` layout. |
| Archive command | Exposed in this slice | ~20 lines of Rust. Domain already supports it. |
| Delete protection | Delete button disabled with tooltip when records reference account | Domain: accounts with records cannot be safely deleted. UI enforces this. Archive is always allowed (soft-delete). |
| Responsive behavior | Desktop-first (Tauri app). Side menu collapses per `accounts.md` §9. | Primary target is local Tauri mode on desktop. |

## 3. Implementation Layers

### Layer 1: Backend — expose archive_account

**File:** `packages/endpoint-tauri/endpoint-tauri/src/account.rs`

Add a `#[tauri::command] pub async fn archive_account(...)`:
- Takes `id: String`
- Builds `AccountCommand::Archive` with `archived_at = Utc::now()`
- Calls `state.account_service.handle(...)`
- Returns `AccountResponse`

Register in `lib.rs` `register_handlers`.

**Verification:** Tauri app starts. `invoke("archive_account", { id })` sets `archived_at`. Existing account CRUD tests pass.

### Layer 2: Frontend Foundation (shared + endpoint-tauri)

#### 2a. Account model (`packages/shared/app/models/account.ts`)

```ts
export enum AccountType { Asset = "Asset", Liability = "Liability", Equity = "Equity", Income = "Income", Expense = "Expense" }

export interface Account {
  id: string; journal_id: string; parent_id: string | null;
  type: AccountType; name: string; description: string;
  tags: string[]; created_at: string | null;
  last_modified_at: string | null; archived_at: string | null;
}
export interface CreateAccountRequest { journal_id: string; parent_id: string; name: string; description?: string; tags?: string[]; }
export interface UpdateAccountRequest { name?: string; description?: string; tags?: string[]; }
export interface AccountFilter { id?: string; journal_id?: string; parent_id?: string; name?: string; type?: string; tag?: string; fullText?: string; }
export interface AccountFormData { name: string; description: string; tags: string[]; }
```

**Also define the table row type** (account + nested children):

```ts
export interface AccountRow extends Account {
  subRows: AccountRow[];
  depth: number;
}
```

Export from `packages/shared/index.ts`.

#### 2b. AccountClient interface (`packages/shared/app/clients/account-client.ts`)

```ts
export interface AccountClient {
  create(request: CreateAccountRequest): Promise<Account>;
  get(id: string): Promise<Account>;
  list(filter?: AccountFilter): Promise<Account[]>;
  update(id: string, request: UpdateAccountRequest): Promise<Account>;
  delete(id: string): Promise<void>;
  archive(id: string): Promise<Account>;
}
declare module "#app" { interface NuxtApp { $accountClient: AccountClient; } }
```

Export from `packages/shared/index.ts`.

#### 2c. TauriAccountClient (`packages/endpoint-tauri/app/clients/account-client.ts`)

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AccountClient, Account, CreateAccountRequest, UpdateAccountRequest, AccountFilter } from "@white-rabbit/shared";

export class TauriAccountClient implements AccountClient {
  async create(request: CreateAccountRequest): Promise<Account> { return invoke("create_account", { request }); }
  async get(id: string): Promise<Account> { return invoke("get_account", { id }); }
  async list(filter?: AccountFilter): Promise<Account[]> { return invoke("list_accounts", { filter: filter ?? {} }); }
  async update(id: string, request: UpdateAccountRequest): Promise<Account> { return invoke("update_account", { id, request }); }
  async delete(id: string): Promise<void> { return invoke("delete_account", { id }); }
  async archive(id: string): Promise<Account> { return invoke("archive_account", { id }); }
}
```

#### 2d. Account client plugin (`packages/endpoint-tauri/app/plugins/account-client.ts`)

```ts
import { TauriAccountClient } from "~/clients";
export default defineNuxtPlugin(() => ({
  provide: { accountClient: new TauriAccountClient() },
}));
```

#### 2e. useAccountClient composable (`packages/shared/app/composables/useAccountClient.ts`)

```ts
export function useAccountClient(): AccountClient {
  return useNuxtApp().$accountClient as AccountClient;
}
```

#### 2f. Account composables (`packages/shared/app/composables/account.ts`)

```ts
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
```

#### 2g. useAccountTree composable (`packages/shared/app/composables/account.ts`, same file)

Transforms a flat `Account[]` into nested `AccountRow[]` for the table.

```ts
export function useAccountTree(accounts: Ref<Account[]>, showArchived: Ref<boolean>) {
  const tree = computed<AccountRow[]>(() => {
    const filtered = showArchived.value
      ? accounts.value
      : accounts.value.filter(a => !a.archived_at);

    // 1. Group by type into 5 buckets
    // 2. Within each bucket, index by parent_id
    // 3. Recursively attach children → subRows
    // 4. Compute depth
    return buildAccountRows(filtered);
  });
  return { tree };
}
```

Algorithm:
1. Partition flat list by `type` into 5 buckets.
2. Within each bucket, index accounts by `parent_id` into a `Map<string | null, Account[]>`.
3. Start from `parent_id === null` (the 5 roots). For each root, recursively attach children as `subRows`, calculating `depth = parent.depth + 1`.
4. Return the 5 roots as the top-level array.

Flat accounts from backend → nested `AccountRow[]` with `subRows` and `depth`.

#### 2h. layouts/journal.vue (`packages/shared/app/layouts/journal.vue`)

Nuxt layout wrapping `default.vue` with a journal-scoped side menu.

```vue
<script setup lang="ts">
const route = useRoute();
const journalId = computed(() => route.params.id as string);
const { data: journal } = useJournal(journalId);
</script>
<template>
  <NuxtLayout name="default">
    <div class="flex">
      <aside class="w-48 shrink-0 border-r border-outline-variant p-3">
        <nav>
          <NuxtLink :to="`/journals/${journalId}`" class="block">Dashboard</NuxtLink>
          <NuxtLink :to="`/journals/${journalId}/accounts`" class="block">Accounts</NuxtLink>
          <span class="block text-on-surface-variant/40">Records</span>
          <span class="block text-on-surface-variant/40">Reports</span>
        </nav>
      </aside>
      <main class="flex-1 p-4">
        <slot />
      </main>
    </div>
  </NuxtLayout>
</template>
```

Side menu uses `NuxtLink` for active items, muted `<span>` for disabled placeholders.

#### 2i. useCurrentJournal composable (`packages/shared/app/composables/useCurrentJournal.ts`)

```ts
export function useCurrentJournal() {
  const route = useRoute();
  const journalId = computed(() => route.params.id as string);
  const { data: journal, refresh, status } = useJournal(journalId);
  return { journal, journalId, refresh, status };
}
```

### Layer 3: Enhance AppDataTable — expanding support

**File:** `packages/shared/app/components/ui/AppDataTable.vue`

Add optional expanding support without breaking existing usage. New optional props:

```ts
const props = defineProps<{
  data: TData[];
  columns: ColumnDef<TData, unknown>[];
  // New — expanding support
  getSubRows?: (row: TData) => TData[];
  enableExpanding?: boolean;
}>();
```

When `enableExpanding` is true and `getSubRows` is provided:

1. Add `expanded` state:
```ts
const expanded = ref<ExpandedState>({});
```

2. Add to table options:
```ts
getSubRows: props.getSubRows,
getExpandedRowModel: getExpandedRowModel(),
state: {
  ...(props.enableExpanding ? { expanded: expanded.value } : {}),
},
onExpandedChange: props.enableExpanding
  ? (updater) => { expanded.value = typeof updater === "function" ? updater(expanded.value) : updater; }
  : undefined,
```

Import `getExpandedRowModel` from `@tanstack/vue-table`.

**Important:** Expanding rows renders sub-rows via `table.getRowModel().rows` (which already includes expanded children). The expand/collapse toggle UI is added by the consumer in a column definition, NOT baked into AppDataTable. AppDataTable only provides the expanded row model infrastructure.

### Layer 4: Accounts Page

**Route:** `app/pages/journals/[id]/accounts.vue`

**Layout:** `journal`

#### 4a. Page component (`pages/journals/[id]/accounts.vue`)

Orchestrator. Owns dialog visibility state, selected account, search filter, show-archived toggle.

```
Data:   useCurrentJournal()         → journal name
        useAccounts(journalId)      → flat Account[]
        useAccountTree(accounts, showArchived) → AccountRow[] (nested)

State:  searchQuery, showArchived, selectedAccount, dialogMode
```

Renders: page title "Accounts", search input, show-archived toggle, `AccountTable`, create/edit/archive/delete dialogs.

#### 4b. AccountTable.vue (`app/components/AccountTable.vue`)

Uses the enhanced `AppDataTable` with expanding enabled.

```vue
<script setup lang="ts">
import { h } from "vue";
import type { AccountRow, Account } from "../models";
import AppDataTable from "./ui/AppDataTable.vue";
import AppButton from "./ui/AppButton.vue";
import AppIcon from "./ui/AppIcon.vue";
import AppChip from "./ui/AppChip.vue";
import AppTooltip from "./ui/AppTooltip.vue";
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
      return h("div", { class: "flex items-center gap-2" }, [
        // Expand/collapse chevron
        row.getCanExpand()
          ? h(AppButton, {
              variant: "ghost",
              size: "sm",
              onClick: row.getToggleExpandedHandler(),
            }, () => h(AppIcon, {
              icon: row.getIsExpanded() ? "lucide:chevron-down" : "lucide:chevron-right",
              size: "sm",
            }))
          : h("span", { class: "w-6" }), // placeholder for alignment

        // Type icon
        h(AppIcon, {
          icon: TYPE_ICONS[row.original.type] ?? "lucide:folder",
          size: "sm",
        }),

        // Account name (indented by depth)
        h("span", {
          class: row.original.depth > 0 ? "text-on-surface" : "font-semibold text-on-surface",
          style: { paddingLeft: `${row.original.depth * 12}px` },
        }, row.original.name),

        // Archived badge
        row.original.archived_at
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
      return h("div", { class: "flex items-center gap-1" }, [
        h(AppButton, {
          variant: "ghost", size: "sm",
          onClick: () => emit("create", account.id),
        }, () => h(AppIcon, { icon: "lucide:plus", size: "sm" })),

        // Edit — disabled on roots
        isRoot ? null : h(AppButton, {
          variant: "ghost", size: "sm",
          onClick: () => emit("edit", account),
        }, () => h(AppIcon, { icon: "lucide:pencil", size: "sm" })),

        // Archive — disabled on roots and already archived
        if (!isRoot && !account.archived_at) {
          return h(AppButton, {
            variant: "ghost", size: "sm",
            onClick: () => emit("archive", account),
          }, () => h(AppIcon, { icon: "lucide:archive", size: "sm" }));
        }

        // Delete — always visible but disabled if records exist (future)
        // For now, always enabled on non-roots. When records land, check record count.
        isRoot ? null : h(AppButton, {
          variant: "ghost", size: "sm",
          class: "text-error",
          onClick: () => emit("delete", account),
        }, () => h(AppIcon, { icon: "lucide:trash-2", size: "sm" })),
      ]);
    },
  },
];
</script>

<template>
  <AppDataTable
    :data="data"
    :columns="columns"
    :get-sub-rows="(row: AccountRow) => row.subRows"
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

**Key behaviors:**
- Name column: expand chevron + type icon + indent via `paddingLeft` + archived badge. Roots are bold.
- Tags column: `AppChip`s, hidden when empty.
- Actions column: `[+]` (always), `[✎]` (non-root), `[archive]` (non-root, non-archived), `[🗑]` (non-root).
- When records land, the delete button is disabled with an `AppTooltip`: "Cannot delete: N records reference this account."
- Archive is always allowed (soft delete that preserves data).

#### 4c. AccountForm.vue (`app/components/AccountForm.vue`)

Mirrors `JournalForm.vue`. Used for create and edit.

Props: `initial?: { name, description, tags }`, `parentPath?: string` (display-only breadcrumb from parent account name).

Fields: parent path (read-only), type (read-only, inherited from parent), `AppInput` for name, `AppTextarea` for description, `AppTagInput` for tags.

Client-side validation: reserved name check against `["Asset","Liability","Equity","Income","Expense"]`.

Emits: `submit(data: AccountFormData)`, `cancel()`.

#### 4d. AccountArchiveConfirm.vue

Props: `account: Account`, `cascadeCount: number`.

Shows: "Archive account {name}? This will also archive {N} descendant accounts. Archived accounts cannot receive new records, but their history is preserved."

Emits: `confirm()`, `cancel()`.

#### 4e. AccountDeleteConfirm.vue

Props: `account: Account`, `cascadeCount: number`.

Shows: "Delete account {name}? Type the name to confirm." Typed-name gate (pattern from journal delete in `index.vue`).

Emits: `confirm()`, `cancel()`.

### Layer 5: Minimal Dashboard

**Route:** `app/pages/journals/[id].vue`
**Layout:** `journal`

A simple landing page with:

```
┌──────────────────────────────────────────────┐
│  {Journal Name}                              │
│  {description}    [tag1] [tag2]              │
├──────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐      │
│  │  Accounts      │  │  Records       │      │
│  │  12 total      │  │  Coming soon   │      │
│  │  [Manage →]    │  │                │      │
│  └────────────────┘  └────────────────┘      │
│  ┌────────────────┐  ┌────────────────┐      │
│  │  Reports       │  │  Settings      │      │
│  │  Coming soon   │  │  Coming soon   │      │
│  └────────────────┘  └────────────────┘      │
└──────────────────────────────────────────────┘
```

Data: `useCurrentJournal()` + `useAccounts(journalId)`.

Account card links to `/journals/:id/accounts`. Records/Reports/Settings cards are disabled placeholders.

## 4. Component-Dependency Graph

```
layouts/journal.vue
  └─ useJournal(journalId)

useCurrentJournal()
  └─ useJournal(journalId)

pages/journals/[id].vue (dashboard)
  └─ useCurrentJournal()
  └─ useAccounts(journalId)

pages/journals/[id]/accounts.vue
  └─ useCurrentJournal()
  └─ useAccounts(journalId)
  └─ useAccountTree(accounts, showArchived)
  └─ AccountTable
       └─ AppDataTable (enhanced with expanding)
  └─ AccountForm (in dialog)
  └─ AccountArchiveConfirm (in dialog)
  └─ AccountDeleteConfirm (in dialog)

AppDataTable.vue (enhanced)
  └─ @tanstack/vue-table: getExpandedRowModel, ExpandedState
```

## 5. Files Created / Modified

### New files

| File | Package |
|------|---------|
| `app/models/account.ts` | shared |
| `app/clients/account-client.ts` | shared |
| `app/composables/useAccountClient.ts` | shared |
| `app/composables/account.ts` | shared (useAccounts + useAccount + useAccountTree) |
| `app/composables/useCurrentJournal.ts` | shared |
| `app/layouts/journal.vue` | shared |
| `app/components/AccountTable.vue` | shared |
| `app/components/AccountForm.vue` | shared |
| `app/components/AccountArchiveConfirm.vue` | shared |
| `app/components/AccountDeleteConfirm.vue` | shared |
| `app/pages/journals/[id].vue` | shared |
| `app/pages/journals/[id]/accounts.vue` | shared |
| `app/clients/account-client.ts` | endpoint-tauri |
| `app/plugins/account-client.ts` | endpoint-tauri |

### Modified files

| File | Change |
|------|--------|
| `packages/endpoint-tauri/endpoint-tauri/src/account.rs` | Add `archive_account` command |
| `packages/endpoint-tauri/endpoint-tauri/src/lib.rs` | Register `archive_account` handler |
| `packages/shared/app/components/ui/AppDataTable.vue` | Add `getSubRows`, `enableExpanding` props + `getExpandedRowModel` + expanded state |
| `packages/shared/index.ts` | Export account types + `AccountClient` + `AccountRow` |
| `docs/product/pages/accounts.md` | Fix IA/side menu positioning (done) |
| `docs/product/pages/journal.md` | Add phasing note (done) |

## 6. Testing

### Backend
- `archive_account` command sets `archived_at` and persists.
- Existing account CRUD tests in `crates/domain/src/account/test.rs` continue to pass.

### Frontend
- **useAccountTree composable:** Given flat `Account[]`, verify nested `AccountRow[]` has correct `subRows`, `depth`, 5 roots.
- **AccountTable columns:** Verify expand chevron toggles subRows visibility; type icons map correctly per AccountType; indent increases with depth.
- **AccountTable actions:** Roots show only `[+]`. Non-roots show `[✎] [archive] [🗑]`. Archived accounts hide archive button.
- **AccountForm:** Reserved-name validation blocks "Asset", "Liability", etc. Submit emits `AccountFormData`. Cancel emits.
- **AccountDeleteConfirm:** Typed-name gate enables delete only on exact match.
- **AccountArchiveConfirm:** Cascade count displayed correctly.
- **AppDataTable expanding:** Existing table usage (without expanding props) works unchanged.
- **Integration:** Journal card click → dashboard → Accounts link → accounts table renders with expandable tree.
- **Existing tests:** `JournalCard.test.ts`, `JournalForm.test.ts`, UI primitives tests continue to pass.

## 7. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Nuxt dynamic route `[id]` conflicts with existing `/journals/:id` (currently 404s) | The dynamic route page IS the new page. No conflict. |
| `layouts/journal.vue` wrapping `default.vue` causes double header | Use `<NuxtLayout name="default">` inside `journal.vue` to nest layouts. |
| `AppDataTable` expanding breaks existing table usage | Expanding is opt-in via `enableExpanding` prop. Without it, behavior is unchanged. |
| Account tree performance with hundreds of accounts | TanStack Table handles 500+ rows with subRows via `getExpandedRowModel`. Collapsed rows skip rendering children. |
| `archive_account` not idempotent | Domain validates: cannot archive already-archived. Errors → error banner in UI. |
| Delete of accounts with records (future) | Delete button disabled with tooltip. When records land, check `recordCount > 0`. |
| Expanding state lost on data refresh | `expanded` state tracked by row ID. After refresh, same IDs remain expanded if still present. |
