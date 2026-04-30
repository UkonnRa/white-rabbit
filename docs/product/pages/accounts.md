# Accounts Page

The Accounts page is the account-tree workspace for a single journal
(`/journals/:id/accounts`). It is a drill-down from the Journal Dashboard
(`/journals/:id`) for focused account CRUD work (create, edit, archive,
delete, batch import).

## 1. Goal

Let the user see and shape the account tree for one journal: the 5
immutable root accounts (Asset, Liability, Equity, Income, Expense) and
every user-created account beneath them.

An Account is a node in a type-homogeneous tree — its type is inherited
from its parent and never changes, and its position in the tree is fixed
at creation. See `docs/product/features.md` §2 for the full domain rules.

The page must stay readable from small trees (5 roots + a handful of
leaves) up to large trees (hundreds of nested accounts across multiple
levels), and make the hierarchy legible at a glance.

## 2. Use Cases

| #    | Actor | Action                                   | Outcome                                                               |
| ---- | ----- | ---------------------------------------- | --------------------------------------------------------------------- |
| UC-1 | User  | Opens `/journals/:id/accounts`           | Sees the full account tree with the 5 roots expanded                  |
| UC-2 | User  | Expands / collapses a subtree            | Children show / hide; state persists within the session               |
| UC-3 | User  | Clicks "Add child" on an account         | Create dialog opens with that account preset as the parent            |
| UC-4 | User  | Edits an account's name/description/tags | Edit dialog opens; submit updates the node in place                   |
| UC-5 | User  | Archives an account                      | Confirmation dialog warns about cascade → archives node + descendants |
| UC-6 | User  | Un-archives an account                   | _(Backend gap — see §11)_                                             |
| UC-7 | User  | Deletes an account                       | Confirmation dialog warns about cascade → deletes node + descendants  |
| UC-8 | User  | Searches / filters by name or tag        | Tree filters to matching nodes, keeping ancestors visible             |
| UC-9 | User  | Toggles "Show archived"                  | Archived nodes appear greyed out or hidden entirely                   |

## 3. Information Architecture

```text
/                           ← Journals list
/journals/:id               ← Journal Dashboard
                                (stats + nav cards, see journal.md)
/journals/:id/accounts      ← Accounts page (this doc) — full tree CRUD
/journals/:id/records       ← Records list (future)
/journals/:id/reports/...   ← Reports (future)
```

The Accounts page is a sub-page of the Journal Dashboard. The journal
context comes from the route parameter `:id` and is held by the
journal-scoped layout (see §4.3).

## 4. Layout Design

### 4.1 Page Structure

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  App Header (global)   ·   breadcrumb: White Rabbit > {Journal} > Accounts │
├──────────────┬───────────────────────────────────────────────────────────┤
│              │                                                           │
│ Side menu    │  Page title             Search    [Show archived]         │
│              │  "Accounts"             ┌──────┐  [ ]                     │
│ · Dashboard  │                         │filter│                          │
│ · Accounts ★ │                         └──────┘                          │
│ · Records    │                                                           │
│ · Reports    │  ┌──────────────────────────────────────────────────────┐ │
│              │  │ Name               │ Tags       │ Commodities │ Recs  │ │
│              │  ├────────────────────┼────────────┼─────────────┼───────┤ │
│              │  │ ▾ 📄 Asset         │            │             │       │ │
│              │  │    ▸ 📄 Bank       │ [checking] │ 2 USD·EUR   │ 142   │ │
│              │  │       📄 Checking  │            │ USD         │ 120   │ │
│              │  │       📄 Savings   │            │ EUR         │  22   │ │
│              │  │    📄 Cash         │            │ USD         │  45   │ │
│              │  │ ▸ 📄 Liability     │            │             │       │ │
│              │  │ ▸ 📄 Equity        │            │             │       │ │
│              │  │    📄 Opening      │ [system]   │ USD         │   1   │ │
│              │  │ ▸ 📄 Income        │            │             │       │ │
│              │  │ ▸ 📄 Expense       │            │             │       │ │
│              │  └──────────────────────────────────────────────────────┘ │
│              │                                                           │
├──────────────┴───────────────────────────────────────────────────────────┤
│  Footer (global)                                                         │
└──────────────────────────────────────────────────────────────────────────┘
```

- **Table view with expandable tree rows** — the 5 roots are always rendered;
  children expand/collapse in-place via chevron. Depth shown via left-padding
  (12px per level). Future columns (Commodities, Records count) shown as
  placeholders until records land.
- **Per-row action slot** in the last column: `[+] add child`, `[✎] edit`,
  `[archive]`, `[🗑] delete`. Root rows show only `[+]`.
- **Expand/collapse** — clicking the chevron toggles child rows via TanStack
  Table's `getExpandedRowModel`. Default: all 5 roots expanded on first visit.
- **Built on `AppDataTable`** with expanding support — the same recipe-driven
  table component used elsewhere, enhanced with `getSubRows` + expanded state.

### 4.2 App Header and Breadcrumb

The global header from the default layout is retained. On journal
sub-pages the header gains a breadcrumb rendered in the center zone:

`White Rabbit > {Journal name} > Accounts`

Each segment links back to its page (`/` and `/journals/:id`).

### 4.3 Side Menu Strategy

This is the first page that needs the side menu. It must be introduced
with this slice.

| Page                | Side menu? | Content                          |
| ------------------- | ---------- | -------------------------------- |
| `/` (Journals)      | **No**     | Uses `layouts/default.vue`       |
| `/journals/:id/...` | **Yes**    | Uses `layouts/journal.vue` (new) |

`layouts/journal.vue` wraps `layouts/default.vue`'s header/footer and
adds the side menu. Side-menu items for this slice:

| Item      | Route                    | Status in this slice               |
| --------- | ------------------------ | ---------------------------------- |
| Dashboard | `/journals/:id`          | Stats + nav cards (see journal.md) |
| Accounts  | `/journals/:id/accounts` | **This page**                      |
| Records   | `/journals/:id/records`  | Disabled placeholder               |
| Reports   | `/journals/:id/reports`  | Disabled placeholder               |

The side menu is journal-scoped — it reads `journalId` from the route
and exposes it to the page via a `useCurrentJournal()` composable (new).

## 5. Account Table — Detailed Design

The accounts are rendered as an **expandable table** using `AppDataTable`
with TanStack Table's [expanding](https://tanstack.com/table/latest/docs/guide/expanding)
feature. The flat account list from the backend is transformed into nested
rows (`subRows`) client-side. Root accounts (parent_id = null) form the
top-level rows; all other accounts are nested under their parent via `subRows`.

### 5.1 Column Definitions

| Column  | Data source    | Format                                                                                                                                                                                          |
| ------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Name    | `account`      | Expand chevron + type icon (per `AccountType`) + account name (left-padded by depth × 12px) + "Archived" badge (if `archived_at`). Roots are bold; leaf accounts (no children) show no chevron. |
| Tags    | `account.tags` | `AppChip` components. Hidden when empty.                                                                                                                                                        |
| Actions | —              | `[+] add child` (always), `[✎] edit` (non-root), `[archive]` (non-root, non-archived), `[🗑] delete` (non-root). Delete disabled with tooltip when records reference this account (future).     |

Future columns (gated on records):

- **Commodities** — count of unique currency/commodity symbols across
  records posting to this account. Shown as `"—"` initially.
- **Records** — count of records posting to this account.
  Shown as `"—"` initially.

### 5.2 Row Interactions

| Interaction          | Behavior                                                                                                                                              |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Click expand chevron | Toggles child rows visibility via `row.getToggleExpandedHandler()`.                                                                                   |
| Click `[+]`          | Open create dialog with this account preset as parent.                                                                                                |
| Click `[✎]` edit     | Open edit dialog. Disabled on the 5 roots.                                                                                                            |
| Click `[archive]`    | Open archive-confirm dialog. Disabled on roots and already-archived.                                                                                  |
| Click `[🗑]` delete  | Open delete-confirm dialog. Disabled on roots. In future: disabled with tooltip "Cannot delete: N records reference this account" when records exist. |
| Click column header  | Sort by that column (TanStack sorting).                                                                                                               |

### 5.3 Root Account Treatment

The 5 roots (Asset, Liability, Equity, Income, Expense) are top-level
rows (parent_id = null) with:

- Bold font weight.
- Type-colored left-border accent (asset/income = green family,
  liability/expense = red family, equity = neutral).
- Only `[+]` add-child action; no edit/archive/delete.
- Cannot be collapsed out of existence — roots always visible.
- Expanded by default on first visit.

### 5.4 Archived Accounts

- Render with reduced opacity and a muted "Archived" badge after the name.
- Hidden by default; the "Show archived" toggle above the table reveals them.
- The archive action button is hidden on already-archived rows.
- Do not appear in the create-dialog parent picker or any future
  record-entry account picker.

### 5.5 Empty State

The table is never fully empty — the 5 roots exist from the moment a
journal is created. When no user-created accounts exist, the 5 root
rows render with no children (no expand chevron), and the empty state
is shown as:

```text
No accounts found. Create one with the [+] button on a root account.
```

### 5.6 Data Transformation

Flat `Account[]` from `AccountClient.list()` is transformed into nested
`AccountRow[]` client-side:

1. Partition the flat list by `type` into 5 buckets.
2. Within each bucket, index by `parent_id`.
3. Recursively attach children as `subRows`, computing `depth` per row.
4. The 5 root accounts (parent_id = null) form the top-level rows.

TanStack Table's `getSubRows` returns `row.subRows` for each row.

## 6. Create / Edit / Archive / Delete — Interaction Model

### 6.1 Create

Triggered by any `[+]` action or the "Add account" inline CTA. Opens an
`AppDialog` titled "Add Account". The parent is preset to the node the
user clicked from and displayed as a read-only breadcrumb at the top of
the form (`Asset > Bank > …`). The form fields are:

| Field       | Control        | Rule                                                                               |
| ----------- | -------------- | ---------------------------------------------------------------------------------- |
| Parent      | Read-only path | Preset from the `[+]` row clicked. Displayed as a breadcrumb (`Asset > Bank > …`). |
| Type        | Read-only      | Inherited from parent — displayed for clarity, not editable                        |
| Name        | `AppInput`     | Required, non-empty, must not match reserved root names                            |
| Description | `AppTextarea`  | Optional                                                                           |
| Tags        | `AppTagInput`  | Optional                                                                           |

Reserved-name validation happens client-side for fast feedback and is
re-checked by the domain (`Account::is_reserved_name`) on submit.

### 6.2 Edit

Triggered by the `[✎]` action on any non-root node. Opens the same
dialog pre-filled, titled "Edit Account". Parent and type fields are
shown but **not editable** — the domain forbids moving accounts. If the
user wants to re-organize, they must create a new account and migrate
records manually (document this in a short help text under the parent
field).

### 6.3 Archive

Triggered by `[⊘]` on any non-archived, non-root node. Opens a
confirmation dialog:

```text
Archive account "{name}"?

This will also archive all descendant accounts
({N} accounts total). Archived accounts cannot
receive new records, but their history is preserved.

[Cancel]  [Archive]
```

Cascade preview (`{N}`) is computed client-side from the already-loaded
tree. Confirmation runs `AccountCommand::Archive` with `archived_at = now`.

### 6.4 Delete

Triggered by `[🗑]` on any non-root node. Opens a confirmation dialog
patterned on the journal-delete confirmation — requires typing the
account name to enable the destructive button:

```text
Delete account "{name}"?

This permanently deletes "{name}" and {N} descendant
accounts. Any records still posting to these accounts
will fail to load. This action cannot be undone.

Type "{name}" to confirm:  [____________]

[Cancel]  [Delete]   ← disabled until confirmation text matches
```

## 7. Component Breakdown

### 7.1 Shared Project-Level Components

All required primitives already exist under `app/components/ui/`
(`AppButton`, `AppIcon`, `AppChip`, `AppInput`, `AppTextarea`,
`AppTagInput`, `AppDialog`, `AppMenu`, `AppTooltip`). **No new shared
components are needed for this page.**

`AppDataTable` is enhanced in this slice with optional expanding support
(`getSubRows`, `enableExpanding` props, `getExpandedRowModel`). Existing
table usage (without expanding) is unaffected.

### 7.2 Page-Internal Components

Live in `app/components/` (not `ui/`) — specific to the Accounts page.

| Component                   | Responsibility                                                                                                        |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `AccountTable.vue`          | Renders accounts as an expandable TanStack Table. Uses enhanced `AppDataTable`. Defines columns, owns expanded state. |
| `AccountForm.vue`           | Create/edit form. Mirrors `JournalForm.vue`. Parent path displayed as read-only breadcrumb. Emits `submit`, `cancel`. |
| `AccountArchiveConfirm.vue` | Archive confirmation dialog content — name + cascade preview.                                                         |
| `AccountDeleteConfirm.vue`  | Delete confirmation dialog content — name + cascade preview + typed-name gate.                                        |

### 7.3 Layout and Composables (new)

| Item                               | Responsibility                                                                                                           |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `layouts/journal.vue`              | Journal-scoped shell: header breadcrumb + side menu. Reads `journalId` from route and loads the journal once.            |
| `composables/useCurrentJournal.ts` | Exposes the currently-scoped `Journal` and its `id` to any component under the `journal` layout.                         |
| `composables/account.ts`           | `useAccounts(journalId)` — list all accounts for one journal; `useAccount(id)` — single lookup. Built on `useAsyncData`. |
| `composables/useAccountClient.ts`  | Injects the registered `$accountClient`.                                                                                 |
| `clients/account-client.ts`        | `AccountClient` interface — `create / get / list / update / delete / archive`. Declares `$accountClient` on NuxtApp.     |
| `models/account.ts`                | `Account`, `CreateAccountRequest`, `UpdateAccountRequest`, `AccountFilter`, `AccountFormData`, `AccountType` enum.       |

## 8. Filter / Search Behavior

A single search input above the table filters by account name and tags
(client-side — the full tree is already loaded). TanStack Table's
`getFilteredRowModel` handles the filtering.

- Case-insensitive substring match on `name` OR any `tag`.
- **Ancestors of matches stay visible** even if they don't match — the
  user must see the tree context. Achieved via `filterFromLeafRows: true`
  in the table options so a parent is included if any child matches.
- When the tree has ≥20 accounts the search input is always visible;
  below that it may be tucked behind a search icon.
- A "Show archived" toggle sits next to the search input (see §5.4).
- When the filter is active and produces zero matches: show "No
  accounts match your search" in the table empty slot.

## 9. Responsive Behavior

| Breakpoint          | Layout                                                                                                                                |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| < 640px (mobile)    | Side menu collapses to a drawer (hamburger). Table takes full width. Action slot collapses into a single overflow `[⋯]` menu per row. |
| 640–1024px (tablet) | Side menu as icon rail, expands on hover. Actions inline.                                                                             |
| > 1024px (desktop)  | Side menu fully expanded. Actions inline.                                                                                             |

## 10. Page Route and Data Flow

```text
Route:      /journals/:id/accounts
Layout:     journal (new)
Data:       useCurrentJournal()            → Journal (loaded by layout)
            useAccounts(journalId)         → AccountClient.list({ journalId })
Mutations:  AccountClient.create / update / delete / archive → refresh()
```

The tree is built client-side from the flat list returned by
`list({ journalId })`. Building the tree:

1. Partition the flat list by `type` into 5 buckets.
2. Within each bucket, index by `parent_id`.
3. Recursively attach children to each account.

This avoids a separate "tree" endpoint and lets the same list power
search, filters, and the parent picker.

## 11. Known Backend Gaps

This spec assumes the following work lands alongside or before the UI.

| Gap                                                                                                                                                                                       | Where                                  |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| `archive_account` Tauri command is **exposed in the accounts-page slice**.                                                                                                                | endpoint-tauri                         |
| Un-archive is **not in the domain** at all — `AccountCommand::Archive` only sets `archived_at`, and there is no `Unarchive` command. Treat un-archive as a future item; UC-6 is deferred. | `crates/domain/src/account/command.rs` |
| No "move account" operation exists and none is planned (tree structure is immutable per `features.md` §2). The UI must never offer one.                                                   | domain                                 |
| No currency / commodity annotation on accounts yet (noted in `features.md` §2 as planned). The tree row has space reserved but renders nothing for it in this slice.                      | domain                                 |

## 12. Future Considerations

- **Bulk import** — the domain already supports `AccountCommand::Batch`.
  A CSV / beancount-format import UI is a clear future slice, probably
  as its own dialog triggered from a page-level menu.
- **Commodities column** — once Records exist, show the count of unique
  currencies/commodities per account in a dedicated column.
- **Records column** — show the number of records posting to each account.
  Enables the delete guard: disable delete when record count > 0.
- **Per-account balance preview** — once Records exist, show a running
  balance column. This requires a read model (balances are derived from
  records).
- **Drag to reorder siblings** — cheap to add on top of this design if
  the domain later grows a `display_order` field. Not blocking.
- **Currency chip** — once the currency annotation lands, render a
  small currency code next to the name (`Checking · USD`).
