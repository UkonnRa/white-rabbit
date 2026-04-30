# Journal Dashboard

The Journal Dashboard is the primary workspace for a single journal
(`/journals/:id`). It is the page the user lands on after clicking a
journal card and serves as the "home" for all work within that journal.

## 1. Goal

Give the user a Fava-style single-page overview of their journal's
financial state: what accounts exist (tree), what transactions have been
recorded (register), and what the numbers say (reports). Everything the
user needs to understand and operate a journal is visible on this one
screen without navigating to sub-pages.

The dashboard surfaces the three domain entities that define a journal:

- **Account tree** — the structural backbone (`docs/product/features.md` §2).
- **Record register** — all financial events (`docs/product/features.md` §3).
- **Reports** — derived views answering questions
  (`docs/product/features.md` §5).

Sub-pages (`/journals/:id/accounts`, `/journals/:id/records`,
`/journals/:id/reports/...`) are drill-downs for focused work, not
replacements for the dashboard view.

## 2. Use Cases

| #     | Actor | Action                                       | Outcome                                                                |
| ----- | ----- | -------------------------------------------- | ---------------------------------------------------------------------- |
| UC-1  | User  | Opens `/journals/:id`                        | Sees the full account tree, recent records, and summary report widgets |
| UC-2  | User  | Clicks an account in the tree                | Record table filters to that account; tree highlights the selection    |
| UC-3  | User  | Clicks "Add record"                          | Inline record-entry form opens (or dialog); posts to selected accounts |
| UC-4  | User  | Edits a record                               | Record row expands or dialog opens; inline edit for fields             |
| UC-5  | User  | Deletes a record                             | Confirmation → record removed; tree balances and reports refresh       |
| UC-6  | User  | Filters records by date range / tags / payee | Table narrows; report widgets reflect filtered data                    |
| UC-7  | User  | Navigates to Accounts sub-page               | Link in tree toolbar → `/journals/:id/accounts` for deep tree editing  |
| UC-8  | User  | Views a report in a widget                   | Balance sheet, income statement, or net worth chart renders in-place   |
| UC-9  | User  | Expands a report widget to full page         | Navigates to `/journals/:id/reports/{report}` for detail               |
| UC-10 | User  | Imports records (beancount / CSV)            | Import dialog → batch command → tree and table refresh                 |

## 3. Information Architecture

```text
/                           ← Journals list (journals.md)
/journals/:id               ← Journal Dashboard (this doc) ★ primary workspace
/journals/:id/accounts      ← Accounts page (accounts.md) — deep tree CRUD
/journals/:id/records       ← Records list (future) — flat table with advanced filters
/journals/:id/reports/...   ← Full-page reports (future) — each report expanded
```

The dashboard is the **center of gravity** for journal operations. Sub-pages
exist for focused, deep work that needs more screen real estate or
specialized interaction patterns (e.g., bulk account operations, advanced
report filtering).

**Implementation phasing:** The initial slice ships a minimal dashboard
(stats + nav cards — see `docs/superpowers/specs/` for the implementation
plan). The full three-panel layout (tree + register + reports) arrives in
subsequent slices when records and reports are implemented.

The existing `accounts.md` should be updated to reflect this:
its first sentence currently positions the Accounts page as "the first
journal-scoped page a user reaches." That role belongs to the dashboard.

## 4. Layout Design

### 4.1 Page Structure

The dashboard uses a **three-panel layout** inspired by Fava but adapted
for a desktop application with drag-resizable panels.

```text
┌──────────────────────────────────────────────────────────────────────┐
│  App Header (global)   ·   breadcrumb: White Rabbit > {Journal Name} │
├────────────┬──────────────────────────┬──────────────────────────────┤
│            │                          │  Reports                     │
│ Side menu  │  Record Register         │  ┌────────────────────────┐  │
│            │                          │  │ Balance Sheet          │  │
│ · Dashboard│  [+ Add Record] [Import] │  │ Assets  · 12,340 USD   │  │
│ · Accounts │  ┌──────────────────────┐│  │ Liabilities · 500 USD  │  │
│ · Records  │  │ Date    │ Desc   │…  ││  │ Equity    · 11,840 USD │  │
│ · Reports  │  │ 03-01   │Grocer… │…  ││  └────────────────────────┘  │
│            │  │ 03-02   │Salary  │…  ││                              │
│            │  │ 03-05   │Rent    │…  ││  ┌────────────────────────┐  │
│            │  │ …       │        │…  ││  │ Income Statement       │  │
├────────────┤  └──────────────────────┘│  │ Income  · 5,000 USD    │  │
│            │                          │  │ Expenses · 2,340 USD   │  │
│ Account    │                          │  │ Net      · 2,660 USD   │  │
│ Tree       │                          │  └────────────────────────┘  │
│            │                          │                              │
│ ▸ Asset    │                          │  ┌────────────────────────┐  │
│   · Bank   │                          │  │ Net Worth              │  │
│     · Chk  │                          │  │  ▄▄▄▄▄▄ ▄▄▄▄▄▄        │  │
│   · Cash   │                          │  │  ██████ ██████ ██████  │  │
│ ▸ Liabil.  │                          │  │   Jan    Feb    Mar    │  │
│ ▸ Equity   │                          │  └────────────────────────┘  │
│ ▸ Income   │                          │                              │
│ ▸ Expense  │                          │                              │
│            │                          │                              │
├────────────┴──────────────────────────┴──────────────────────────────┤
│  Footer (global)                                                     │
└──────────────────────────────────────────────────────────────────────┘
```

### 4.2 Panel Overview

| Panel    | Position | Default width | Purpose                                                        |
| -------- | -------- | ------------- | -------------------------------------------------------------- |
| Tree     | Left     | 240px         | Full account tree, read-focused, with balances per account     |
| Register | Center   | flex-grow     | All records in this journal, paginated, filterable             |
| Reports  | Right    | 320px         | Stacked report widgets: balance sheet, income statement, chart |

The Reports panel can be collapsed (toggle button in panel header) to
give the register more space. The Tree panel can be collapsed when the
user needs maximum register width, but defaults to visible.

### 4.3 Panel Labeling

Three collapsible panels, each with a header strip containing the panel
name and a collapse chevron:

```text
[Accounts ▾]    [Register ▾]    [Reports ▾]
```

### 4.4 Side Menu Strategy

The dashboard uses the same `layouts/journal.vue` side menu as all
journal-scoped pages.

| Item      | Route                    | Status                    |
| --------- | ------------------------ | ------------------------- |
| Dashboard | `/journals/:id`          | **This page** (active)    |
| Accounts  | `/journals/:id/accounts` | Link to deep tree editor  |
| Records   | `/journals/:id/records`  | Link to full records page |
| Reports   | `/journals/:id/reports`  | Link to full reports page |

### 4.5 App Header and Breadcrumb

```text
White Rabbit > {Journal name}
```

On the dashboard, the breadcrumb ends at the journal name (no trailing
segment). Sub-pages add their segment (e.g., `> Accounts`).

## 5. Account Tree Panel — Detailed Design

The tree panel shows the **full account tree** (all 5 roots and all
descendants) in a compact, read-optimized form. This is **not** the
account CRUD workspace — that lives at `/journals/:id/accounts`.

### 5.1 Panel Structure

```text
┌─ Accounts ──────────────────── [+ Add] ─┐
│                                         │
│ ▸ Asset                  12,340.00 USD  │
│   · Bank:Checking        10,000.00 USD  │
│   · Bank:Savings          2,000.00 USD  │
│   · Cash                    340.00 USD  │
│ ▸ Liability               -500.00 USD  │
│   · CreditCard             -500.00 USD  │
│ ▾ Equity                -11,840.00 USD  │
│   · OpeningBalances     -11,840.00 USD  │
│ ▸ Income                       0.00 USD │
│ ▸ Expense                      0.00 USD │
│                                         │
│ ─────────────────────────────────────── │
│ Net (checksum):               0.00 USD  │
│                                         │
│ [+ Add account]   [Open full tree →]    │
└─────────────────────────────────────────┘
```

### 5.2 Tree Row Anatomy

```text
▸  ·  Checking           10,000.00 USD
│  │  │                   │
│  │  │                   └─ Balance (right-aligned, derived from records)
│  │  └─ Account name (leaf dots "·" indent by depth)
│  └─ Expand/collapse chevron (hidden if no children)
```

- **Chevron**: `▸` or `▾` — toggles subtree. Root accounts default to
  **collapsed** on first visit (unlike the Accounts sub-page), since the
  register and reports are the primary focus. The user expands what they
  need.
- **Depth indentation**: 12px per level, with a vertical dotted guide line
  at each level for deep trees.
- **Balance**: computed from records, right-aligned. Positive values in
  green, negative in red, zero in muted text. Shown in the journal's
  primary currency when possible; multi-commodity accounts render a
  summarized "2 commodities" chip that expands on hover.
- **Balance column is initially absent** until Records are implemented.
  Before records exist, show `—` or hide the column entirely.

### 5.3 Row Colors by Type

Root accounts and their descendants use type-colored left-border accents:

| Type      | Color family    | Intent             |
| --------- | --------------- | ------------------ |
| Asset     | Green           | "What I own"       |
| Liability | Red/orange      | "What I owe"       |
| Equity    | Neutral/grey    | "Net worth anchor" |
| Income    | Green (lighter) | "Money in"         |
| Expense   | Red (lighter)   | "Money out"        |

The 5 roots are bolded and visually distinct from children.

### 5.4 Tree Interactions

| Interaction                | Behavior                                                                                                                         |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Click a non-root account   | Filters the register to records posting to that account. Tree highlights the selection.                                          |
| Click a root account       | Expands/collapses the root subtree. Does not filter records.                                                                     |
| Click `[+ Add account]`    | Opens create dialog (same `AccountForm` used by accounts.md). Parent defaults to the last-expanded node, or picker if ambiguous. |
| Click `[Open full tree →]` | Navigates to `/journals/:id/accounts` — the full CRUD workspace.                                                                 |
| Hover a row                | Tooltip shows `description`, `created_at`, and a currency/commodity breakdown of the balance.                                    |

### 5.5 Balance Computation Method (Future)

Account balances are computed from records. This is a **read model**
derived from the write-consistent record stream.

Two approaches (implementation decision, not in this spec):

1. **Client-side summation**: load all records, sum postings per account
   in the browser. Works for moderate record counts.
2. **Pre-computed read model**: a `balance` column on Account or a
   dedicated `AccountBalance` read model updated synchronously on record
   write (Phase A handler). Preferred for performance.

The balance column is a future item and is **not blocking** the initial
dashboard. Without it, the tree shows only the account structure with no
numbers — still useful alongside the register.

## 6. Record Register Panel — Detailed Design

The register shows every Record in the journal as a paginated,
filterable table. This is the primary data surface of the dashboard.

### 6.1 Panel Structure

```text
┌─ Register ──── [+ Add Record] [Import] ────────────────────────────┐
│                                                                     │
│ Filters:  [Date range ▼]  [Account ▼]  [Tags]  [Payee ▸]  [Clear] │
│                                                                     │
│ ┌──────┬────────────┬──────────┬───────────────────┬────────┬─────┐│
│ │ Date │ Description│ Account  │ Amount            │ Tags   │     ││
│ ├──────┼────────────┼──────────┼───────────────────┼────────┼─────┤│
│ │03-05 │ Rent       │ Expense  │ 1,200.00 USD      │ [home] │ ✎ 🗑││
│ │      │            │ · Rent  │                   │        │     ││
│ │      │            │ Asset   │ -1,200.00 USD     │        │     ││
│ │      │            │ · Bank  │                   │        │     ││
│ ├──────┼────────────┼──────────┼───────────────────┼────────┼─────┤│
│ │03-02 │ Salary     │ Income  │ -5,000.00 USD     │ [work] │ ✎ 🗑││
│ │      │            │ · Salary│                   │        │     ││
│ │      │            │ Asset   │  5,000.00 USD     │        │     ││
│ │      │            │ · Bank  │                   │        │     ││
│ ├──────┼────────────┼──────────┼───────────────────┼────────┼─────┤│
│ │03-01 │ Groceries  │ Expense │ 42.50 USD         │ [food] │ ✎ 🗑││
│ │      │ ACME Mart  │ · Food  │                   │        │     ││
│ │      │            │ Asset   │ -42.50 USD        │        │     ││
│ │      │            │ · Bank  │                   │        │     ││
│ ├──────┴────────────┴──────────┴───────────────────┴────────┴─────┤│
│ │                        Page 1 of 3   ← →                        ││
│ └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Table Columns

| Column      | Data source                   | Format                                                                                                                              |
| ----------- | ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Date        | `Record.date`                 | `MM-DD` (year implicit in date-range filter). Tooltip shows full date.                                                              |
| Description | `Record.description`          | Primary text; payee shown below in smaller muted text if present.                                                                   |
| Payee       | `Record.payee`                | Shown as subtext under description when non-empty.                                                                                  |
| Postings    | `Record.items` (Transactions) | Each posting on its own line: `account_name` + `amount`. Account names are indented with `·`. Validations render with a `⇔` marker. |
| Amount      | Derived (total of postings)   | Not a separate column; each posting shows its own amount.                                                                           |
| Tags        | `Record.tags`                 | `AppChip` components. Hidden column if no records have tags.                                                                        |
| Actions     | —                             | `[✎]` edit, `[🗑]` delete.                                                                                                          |

### 6.3 Row Design

Each record occupies a **variable-height row** — one line for the date
and description, then one line per posting. This is the beancount way:

```text
2024-03-01 * "Groceries at ACME Mart"  #food
  Expenses:Food          42.50 USD
  Assets:Bank:Checking  -42.50 USD
```

→ In the table:

```text
03-01  Groceries at ACME Mart          [food]  ✎ 🗑
       Expenses:Food          42.50 USD
       Assets:Bank:Checking  -42.50 USD
```

The first line (description) has a slightly heavier font weight. Posting
lines are indented. Clicking a posting account name filters the register
to that account (same as clicking in the tree).

### 6.4 Filter Bar

A row of filter controls above the table:

| Filter     | Control         | Behavior                                                                                                                    |
| ---------- | --------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Date range | Date picker     | "All time" default; presets: "This month", "Last 3 months", "YTD", custom range.                                            |
| Account    | Dropdown search | Type-ahead search over account names. Filters to records whose postings reference that account. Synced with tree selection. |
| Tags       | Multi-select    | Chip input — records must have at least one selected tag.                                                                   |
| Payee      | Text input      | Substring match on `Record.payee`.                                                                                          |
| Search     | Text input      | Substring match on `Record.description`.                                                                                    |

All filters combine with AND. Active filters show a badge count; clicking
`[Clear]` resets all. The filter state lives in the URL query string so
that the page is shareable/bookmarkable.

### 6.5 Pagination

Records are paginated server-side (or client-side for small journals). A
simple paginator at the bottom of the table:

```text
Page 1 of 3   ← 1 2 3 →
```

Default page size: 50 records. Configurable in settings (future).

### 6.6 Empty State

When no records exist in the journal:

```text
┌──────────────────────────────────────────────┐
│                                              │
│         No records yet.                      │
│   Create your first transaction or           │
│   import a beancount file to get started.    │
│                                              │
│   [+ Add Record]   [Import beancount]        │
│                                              │
└──────────────────────────────────────────────┘
```

### 6.7 Record Actions

| Action                 | Behavior                                                                                                           |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `[+ Add Record]`       | Opens a full `RecordForm` dialog. Accounts are selected from a type-ahead picker. Validates balance on submit.     |
| `[Import]`             | Opens import dialog: file picker + format selector (beancount, CSV). Preview table before confirming batch import. |
| `[✎]` edit on a row    | Opens the same `RecordForm` pre-filled.                                                                            |
| `[🗑]` delete on a row | Confirmation dialog → delete.                                                                                      |

The record form (`RecordForm.vue`) supports:

- Date, description, payee, tags (top section).
- Dynamic list of postings (bottom section): account picker, amount
  (`number + unit`), optional price (`@`), optional cost (`{}`).
- Running balance indicator: shows deviation from zero before submit.
- Toggle between Transaction mode and Validation mode.

### 6.8 Selection Sync with Tree

When the user clicks an account in the tree panel, the register
automatically filters to that account. The account filter in the filter
bar updates to reflect this. Clearing the account filter (or clicking the
root in the tree) restores the full register view.

Conversely, clicking a posting's account name in the register highlights
that account in the tree.

This bidirectional sync makes the dashboard feel like one integrated
tool, not two separate widgets sharing a page.

## 7. Reports Panel — Detailed Design

The reports panel is a scrollable column of stacked widgets. Each widget
is a compact, read-only summary of one report type. This is **not** the
full reports page — just a dashboard snapshot.

### 7.1 Panel Structure

```text
┌─ Reports ──────────────────────── [Full reports →] ─┐
│                                                      │
│ ┌─ Balance Sheet ──────────────────────────────────┐ │
│ │ Assets            12,340.00 USD                   │ │
│ │ Liabilities         -500.00 USD                   │ │
│ │ ────────────────────────────────                   │ │
│ │ Equity (Net)      11,840.00 USD                   │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
│ ┌─ Income Statement ───────────────────────────────┐ │
│ │ Income             5,000.00 USD                   │ │
│ │ Expenses          -2,340.00 USD                   │ │
│ │ ────────────────────────────────                   │ │
│ │ Net Income         2,660.00 USD                   │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
│ ┌─ Net Worth ──────────────────────────────────────┐ │
│ │ ▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄                                   │ │
│ │ ██████ ████████ ████████                          │ │
│ │  Jan     Feb     Mar                              │ │
│ │ Last: 11,840.00 USD  ▲ +520 from Feb              │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
│ ┌─ Holdings ───────────────────────────────────────┐ │
│ │ AAPL    10 shares    @ 175.30 USD                 │ │
│ │ VTI     25 shares    @ 242.10 USD                 │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
│ ┌─ Latest Records ─────────────────────────────────┐ │
│ │ 03-05  Rent                    -1,200.00 USD     │ │
│ │ 03-02  Salary                   5,000.00 USD     │ │
│ │ 03-01  Groceries                  -42.50 USD     │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### 7.2 Report Widgets

Each widget is collapsible (chevron toggle) and shows a `[Expand →]`
link that navigates to the full-page version of that report.

| #   | Widget           | Description                                                                                                     | Priority |
| --- | ---------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| 1   | Balance Sheet    | Assets − Liabilities = Equity. Snapshot at journal's latest date.                                               | P0       |
| 2   | Income Statement | Income − Expenses = Net Income, over the current filter's date range.                                           | P0       |
| 3   | Net Worth Chart  | Time-series line/area chart of net worth over time.                                                             | P1       |
| 4   | Holdings         | Current commodity/stock positions with cost basis. Collapses when no commodities exist.                         | P1       |
| 5   | Latest Records   | Last 5 records as a compact list (click navigates to full register). Quick entry point back to recent activity. | P2       |

### 7.3 Widget Behavior

- Widgets observe the **same filter state** as the register. Changing the
  date range or account filter updates all widgets.
- Widgets show a **skeleton loader** while data is computing.
- Widgets that depend on records (all of them) show a distinct empty
  state when zero records exist: "No data yet" with a link to add a
  record.
- Widgets are stacked vertically. The user can collapse individual
  widgets to reduce clutter, and can reorder them (drag handle) in a
  future iteration.

### 7.4 Reports Are a Future Slice

Reports are **not implemented in the domain yet**. The report widgets are
designed here so the dashboard layout accommodates them from the start,
but the initial dashboard ships with:

```text
┌─ Reports ─────────────────────────────────────────┐
│                                                    │
│         Reports coming soon.                       │
│   The balance sheet, income statement,             │
│   and net worth chart will appear here             │
│   once you have records in this journal.           │
│                                                    │
└────────────────────────────────────────────────────┘
```

The panel itself exists (as a placeholder) so the three-column layout is
stable from day one.

## 8. Record Form — Detailed Design

The "Add Record" / "Edit Record" dialog is the most complex form in the
application. It composes multiple dynamic posting lines with real-time
balance validation.

### 8.1 Form Layout

```text
┌─ New Record ──────────────────────────────────────┐
│                                                    │
│ Date:       [2024-03-15    ▾]  (date picker)      │
│ Description:[Groceries at ACME Mart    ]           │
│ Payee:      [ACME Mart                ]            │
│ Tags:       [food] [personal] [+]                  │
│                                                    │
│ ── Postings ────────────────────────────────────── │
│                                                    │
│ Account: [Expenses:Food        ▾]  Amount: [42.50 USD]  [@ …]  [{} …]  [🗑] │
│ Account: [Assets:Bank:Checking ▾]  Amount: [-42.50 USD] [@ …]  [{} …]  [🗑] │
│                                                    │
│ [+ Add posting]                                    │
│                                                    │
│ Balance: ✓ Balanced (0.00 USD)                     │
│                                                    │
│ [Cancel]                              [Save]       │
└────────────────────────────────────────────────────┘
```

### 8.2 Form Fields

| Field        | Control                 | Domain input                 | Validation                                |
| ------------ | ----------------------- | ---------------------------- | ----------------------------------------- |
| Date         | Date picker             | `RecordInput.date`           | Required, defaults to today               |
| Description  | `AppInput`              | `RecordInput.description`    | Required (NonEmpty in domain)             |
| Payee        | `AppInput`              | `RecordInput.payee`          | Optional                                  |
| Tags         | `AppTagInput`           | `RecordInput.tags`           | Optional                                  |
| Postings (N) | Dynamic list            | `RecordInput.items`          | At least 2; balanced sum; all same kind   |
| · Account    | Autocomplete/search     | `RecordItemInput.account_id` | Must belong to this journal; not archived |
| · Amount     | `AppInput`              | `RecordItemInput.amount`     | Format: `number unit` (e.g., `42.50 USD`) |
| · Price (@)  | `AppInput` (collapsed)  | `RecordItemInput.price`      | Same format as amount; optional           |
| · Cost ({})  | Multi-input (collapsed) | `RecordItemInput.cost`       | Price/date/reference; optional            |

### 8.3 Posting Lines

Each posting line is a horizontal row with:

- **Account picker**: type-ahead search over the full account tree. Shows
  `Asset:Bank:Checking` path. Filters as user types.
- **Amount**: `number unit` format. Green when positive, red when
  negative. Validation: must match `number unit` pattern.
- **\[@ …]**: collapsed by default. Expands to a `number unit` input for
  the price annotation.
- **\[{} …]**: collapsed by default. Expands to a cost input
  (price/date/reference). Multiple costs can be added.
- **\[🗑]**: removes this posting line. Disabled when only 2 postings
  remain (need at least 2 for balance).
- `[+ Add posting]` appends a new empty posting line.

### 8.4 Balance Bar

A live indicator at the bottom of the postings section:

```text
Balance: ✓ Balanced (0.00 USD)          ← green, when sum ≈ 0
Balance: ⚠ Unbalanced (+12.50 USD)      ← red, shows deviation
Balance: ⚠ Multiple commodities (USD, EUR) ← amber, mixed currencies
```

Computed in real-time as the user types. Sums all posting amounts,
grouped by unit. If all amounts are in the same unit and sum to zero:
balanced. If same unit but non-zero: show deviation. If multiple units:
warn about mixed commodities.

## 9. Import Dialog

### 9.1 Flow

```text
[Import] → Select file → Preview → Confirm → Batch create
```

1. **File picker**: native file dialog filtered to `.beancount`, `.csv`, `.txt`.
2. **Format detection**: auto-detects beancount vs CSV by first non-comment line.
3. **Preview table**: shows parsed records before importing. Errors
   (malformed lines, missing accounts, unbalanced transactions) are
   flagged in red with inline messages.
4. **Missing accounts**: if a posting references an account that doesn't
   exist in the tree, the preview offers to auto-create it (with a
   checkbox per account). The created account's parent is inferred from
   the hierarchy path.
5. **Confirm**: batch command executes all records + any auto-created
   accounts. Progress bar during processing. Result: "Imported 142 records,
   3 new accounts created."

## 10. Component Breakdown

### 10.1 Shared Project-Level Components

All primitives exist under `app/components/ui/` (`AppButton`, `AppIcon`,
`AppChip`, `AppInput`, `AppTextarea`, `AppTagInput`, `AppDialog`,
`AppCard`, `AppTooltip`, `AppMenu`). **No new shared components are
needed.**

### 10.2 Page-Internal Components

Live in `app/components/` — specific to the Journal Dashboard.

| Component                   | Responsibility                                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `JournalDashboard.vue`      | Page-level orchestrator: three-panel layout, panel collapse state, filter state, selection sync.                   |
| `AccountTreePanel.vue`      | Wrapper for the tree panel: toolbar (`+ Add`, `Open full tree →`), tree rendering, balance footer.                 |
| `AccountTreeRow.vue`        | Single row in the dashboard tree: chevron, name, balance. Read-only variant of `AccountTreeNode` from accounts.md. |
| `RecordRegisterPanel.vue`   | Wrapper for the register: filter bar, table, paginator, empty state.                                               |
| `RecordTable.vue`           | Table with variable-height rows (using TanStack Table or custom implementation).                                   |
| `RecordTableRow.vue`        | Single record row: date, description, expandable postings.                                                         |
| `RecordForm.vue`            | Create/edit record dialog. Dynamic posting lines, live balance bar.                                                |
| `PostingLine.vue`           | Single posting row inside `RecordForm`: account picker, amount, price, cost, delete.                               |
| `RecordFilterBar.vue`       | Horizontal filter controls: date range, account search, tags, payee, search, clear button.                         |
| `RecordImportDialog.vue`    | Import flow: file picker, format detection, preview table, confirm.                                                |
| `ReportsPanel.vue`          | Wrapper for the reports column: placeholder state, widget stack.                                                   |
| `BalanceSheetWidget.vue`    | Compact balance sheet table. (Future — placeholder until reports land.)                                            |
| `IncomeStatementWidget.vue` | Compact income statement table. (Future.)                                                                          |
| `NetWorthChartWidget.vue`   | Net worth time-series chart. (Future.)                                                                             |
| `HoldingsWidget.vue`        | Current commodity positions. (Future.)                                                                             |
| `LatestRecordsWidget.vue`   | Last 5 records as a compact list. (Future or P2.)                                                                  |

### 10.3 Layout and Composables

| Item                                 | Responsibility                                                                               |
| ------------------------------------ | -------------------------------------------------------------------------------------------- |
| `layouts/journal.vue`                | Already planned in accounts.md. Wraps default layout with side menu and journal context.     |
| `composables/useCurrentJournal.ts`   | Already planned in accounts.md. Exposes the current journal.                                 |
| `composables/useRecords.ts`          | `useRecords(journalId, filters)` — paginated record query. Built on `useAsyncData`.          |
| `composables/useRecordClient.ts`     | Injects `$recordClient`.                                                                     |
| `composables/useDashboardFilters.ts` | Reactive filter state (date range, account, tags, payee, search). Syncs to URL query string. |
| `clients/record-client.ts`           | `RecordClient` interface — `create / get / list / update / delete / batch`.                  |
| `models/record.ts`                   | `Record`, `CreateRecordRequest`, `UpdateRecordRequest`, `RecordFilter`.                      |

## 11. Page Route and Data Flow

```text
Route:      /journals/:id
Layout:     journal
Data:       useCurrentJournal()              → Journal (loaded by layout)
            useAccounts(journalId)            → AccountClient.list({ journalId })
            useRecords(journalId, filters)    → RecordClient.list(params)
Mutations:  RecordClient.create / update / delete / batch → refresh()
            AccountClient.create (from dashboard [+ Add account]) → refreshAccounts()
```

### 11.1 Data Loading Strategy

1. **Journal**: loaded once by `layouts/journal.vue` on first journal-scoped
   navigation. Cached for all sub-pages.
2. **Accounts**: loaded once for the tree panel. Shared with the
   record-form account picker (no second fetch). Built into a tree
   client-side (same as accounts.md §10).
3. **Records**: loaded with current filters, paginated. Re-fetched when
   filters change. The `useRecords` composable wraps `useAsyncData` with
   a reactive filter dependency.
4. **Report data**: in the future, report widgets derive their data from
   the same record stream OR from a dedicated report read model. Either
   way, the data flows through composables that the widgets consume.

### 11.2 Filter State in URL

The filter bar writes its state to the URL query string:

```text
/journals/:id?from=2024-01-01&to=2024-12-31&account=abc123&tags=food,home&payee=ACME&search=groceries&page=2
```

This makes the dashboard shareable and bookmarkable with filters applied.
The `useDashboardFilters` composable reads initial state from `route.query`
and writes updates back via `router.replace`.

## 12. Panel Responsiveness

On narrow viewports, the three-panel layout adapts:

| Breakpoint          | Layout                                                                                                                        |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| < 768px (mobile)    | Single column: register fills the screen. Tree and reports become tabs or slide-over drawers accessed via bottom nav buttons. |
| 768–1280px (tablet) | Two panels: tree (collapsible sidebar, not a persistent panel) + register. Reports collapse to a toggleable drawer.           |
| > 1280px (desktop)  | Full three-panel layout as designed. Panels are drag-resizable.                                                               |

On mobile, the filter bar collapses into a single search input with a
`[Filters ▾]` button that opens a bottom sheet with all filter controls.

## 13. Relationship to Sub-Pages

### 13.1 `/journals/:id/accounts` (accounts.md)

The Accounts page is the **deep-editing workspace** for the account tree.
It exists because:

- Creating, editing, archiving, and deleting accounts (especially in bulk)
  deserves full-screen focus and a richer tree interaction model.
- The dashboard tree is read-optimized; it doesn't need inline edit icons
  or archive/delete per row. Those add visual noise to a dashboard.
- Commodity/currency annotation per account (planned in `features.md` §2)
  requires form fields that don't belong on a dashboard summary.

The dashboard links to /accounts via `[Open full tree →]` and `[+ Add
account]` (which opens the form dialog without leaving the dashboard).

**accounts.md must be updated**: its §3 (Information Architecture) and
§4.3 (Side Menu Strategy) currently position Accounts as the first
journal-scoped page. The dashboard replaces that role. Accounts is now a
secondary drill-down page.

### 13.2 `/journals/:id/records`

A full-page records view with advanced querying, multi-select for
batch-delete, and export. The dashboard's register is the "quick view"
— this is the power-user workspace.

### 13.3 `/journals/:id/reports/...`

One full-page route per report type. The dashboard's report widgets are
compact snapshots; these are the full reports with all controls
(period comparison, currency filter, export, drill-down into
constituent accounts).

## 14. Future Considerations

### 14.1 Dashboard Customization

Users could configure which widgets appear, their order, and the default
panel layout. State persisted per journal in localStorage.

### 14.2 Quick Entry Mode

A floating action button or keyboard shortcut (`Ctrl+N`) opens a minimal
record-entry form directly from the dashboard, bypassing the dialog.

### 14.3 Account Balance Trends

Mini sparkline charts next to account balances in the tree, showing the
balance trend over the last 30/90 days.

### 14.4 Keyboard Navigation

- `J`/`K` — navigate records (Vim-style).
- `E` — edit selected record.
- `N` — new record.
- `F` — focus filter bar.
- `/` — focus search input.

### 14.5 Undo / Redo

Command history per journal session. Undo a record delete, revert an
account rename, etc. Requires a command-log infrastructure (out of scope
for initial release but architected for).
