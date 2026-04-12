# Journals Page

The Journals page is the application home page (`/`). It is the first screen
every user sees after launch.

## 1. Goal

Give the user immediate orientation: which ledgers exist, what is their
recent state, and a fast path to open one or create a new one.

A Journal is a thin container — `{id, name, description, tags, timestamps}`.
It carries no financial numbers of its own. Most users will have 1–5
journals (personal, business, household, travel, etc.). The page design must
work well for 1 journal and scale gracefully to ~20.

## 2. Use Cases

| #    | Actor | Action                                  | Outcome                                                 |
| ---- | ----- | --------------------------------------- | ------------------------------------------------------- |
| UC-1 | User  | Opens the app                           | Sees all journals with summary metadata                 |
| UC-2 | User  | Clicks a journal card                   | Navigates to that journal's dashboard (`/journals/:id`) |
| UC-3 | User  | Clicks "New Journal"                    | Inline creation form appears (no page navigation)       |
| UC-4 | User  | Edits a journal's name/description/tags | Inline edit on the card, or a dialog                    |
| UC-5 | User  | Deletes a journal                       | Confirmation dialog → journal removed                   |
| UC-6 | User  | Searches/filters journals               | Cards filter by name or tag (useful at 5+ journals)     |

## 3. Information Architecture

```text
/                     ← Journals page (this doc)
/journals/:id         ← Journal dashboard (accounts, records, reports)
/journals/:id/...     ← Sub-pages: account tree, record list, reports
/color-demo           ← Dev-only: theme color demo
```

The Journals page is the **only** page without a journal context. All other
pages operate within a selected journal.

## 4. Layout Design

### 4.1 Page Structure

```text
┌─────────────────────────────────────────────────────┐
│  App Header (global)                                │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Page Title          Search        [+ New Journal]  │
│  "Journals"          ┌──────┐                       │
│                      │filter│                       │
│                      └──────┘                       │
│                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │  Journal A   │  │  Journal B   │  │  Journal C  │ │
│  │  desc…       │  │  desc…       │  │  desc…      │ │
│  │  tags        │  │  tags        │  │  tags       │ │
│  │  updated …   │  │  updated …   │  │  updated …  │ │
│  │         ✎ 🗑 │  │         ✎ 🗑 │  │        ✎ 🗑 │ │
│  └─────────────┘  └─────────────┘  └────────────┘  │
│                                                     │
├─────────────────────────────────────────────────────┤
│  Footer (global)                                    │
└─────────────────────────────────────────────────────┘
```

- **Cards in a responsive grid**: 1 column on mobile, 2 on medium, 3 on
  large screens.
- **No side menu on this page.** The side menu is a journal-scoped
  navigation tool (accounts, records, reports). On the Journals page there
  is no journal context, so the side menu has nothing to show. The hamburger
  button is either hidden or disabled.
- **Empty state**: When zero journals exist, show a centered illustration or
  message with a single "Create your first journal" button.

### 4.2 App Header (Global — All Pages)

The header is defined in the default layout and is consistent across all
pages. It contains:

| Zone   | Content                                                     | Notes                                                    |
| ------ | ----------------------------------------------------------- | -------------------------------------------------------- |
| Left   | App logo / name ("White Rabbit")                            | Links back to `/` (Journals page)                        |
| Center | _(empty on Journals page; breadcrumb on journal sub-pages)_ |                                                          |
| Right  | Theme switcher, seed color picker, dark mode toggle         | Dev/settings controls; may move to a settings page later |

On journal sub-pages (`/journals/:id/...`), the header gains a breadcrumb:
`White Rabbit > Journal Name > Accounts`.

### 4.3 Side Menu Strategy

| Page                        | Side menu? | Content                                                   |
| --------------------------- | ---------- | --------------------------------------------------------- |
| `/` (Journals)              | **No**     | No journal context → nothing to navigate                  |
| `/journals/:id` (Dashboard) | **Yes**    | Journal-scoped nav: Dashboard, Accounts, Records, Reports |
| `/journals/:id/accounts`    | **Yes**    | Same journal-scoped nav                                   |
| `/color-demo`               | **No**     | Dev page, no domain context                               |

The side menu is not a global element — it is a journal-scoped navigation
that only appears when the user is inside a journal. This avoids an empty
or confusing drawer on the Journals page.

Implementation: the default layout provides the header and footer.
Journal sub-pages use a nested layout (`layouts/journal.vue`) that adds
the side menu.

## 5. Journal Card — Detailed Design

Each journal renders as a card (`AppCard`) with the following content:

```text
┌──────────────────────────────────────┐
│  Journal Name                    ✎ 🗑│
│                                      │
│  Description text (2 lines max,      │
│  overflow ellipsis)                  │
│                                      │
│  [tag1] [tag2] [tag3]               │
│                                      │
│  Created: 2026-01-15                 │
│  Last updated: 2 days ago            │
└──────────────────────────────────────┘
```

### 5.1 Card Interactions

| Interaction            | Behavior                                                                                                                       |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Click card body        | Navigate to `/journals/:id`                                                                                                    |
| Click edit icon (✎)    | Opens edit mode: name/description/tags become editable inline, or opens a dialog. Stops event propagation (does not navigate). |
| Click delete icon (🗑) | Opens confirmation dialog. Stops propagation.                                                                                  |
| Hover                  | Subtle elevation or border highlight (theme-dependent: MD3 elevation shift, Tailwind border-primary)                           |

### 5.2 Card Content Mapping

| Visual element | Data source                | Notes                                                                    |
| -------------- | -------------------------- | ------------------------------------------------------------------------ |
| Name           | `Journal.name`             | Required, `NonEmpty<String>` in domain                                   |
| Description    | `Journal.description`      | Optional, may be empty — show placeholder text "No description"          |
| Tags           | `Journal.tags`             | Rendered as `AppChip` components. May be empty — hide section if no tags |
| Created date   | `Journal.created_at`       | Formatted as relative ("3 days ago") or absolute date                    |
| Last updated   | `Journal.last_modified_at` | Relative time; falls back to created_at if never modified                |

### 5.3 Empty State

When `journals.length === 0`:

```text
┌──────────────────────────────────────┐
│                                      │
│         No journals yet.             │
│   Create your first ledger to        │
│   start tracking finances.           │
│                                      │
│       [+ Create Journal]             │
│                                      │
└──────────────────────────────────────┘
```

Centered vertically in the content area. Single CTA button.

## 6. Create / Edit Journal — Interaction Model

### 6.1 Create

Clicking "+ New Journal" opens a **dialog** (`AppDialog`) with the journal
form, not inline in the grid. Rationale:

- The card grid is a browse surface; inserting an editable form card
  breaks the visual rhythm.
- A dialog provides clear modal focus for a low-frequency action (users
  create journals rarely).
- The form is identical to the edit form — one component, two modes.

### 6.2 Edit

Clicking the edit icon on a card opens the **same dialog** pre-filled with
the journal's current data. The dialog title shows "Edit Journal" instead
of "New Journal".

### 6.3 Delete

Clicking the delete icon opens a **confirmation dialog** with the journal
name and a warning that all accounts and records within will be deleted.
Two buttons: "Cancel" (default focus) and "Delete" (destructive style).

## 7. Component Breakdown

### 7.1 Shared Project-Level Components (already exist or to be created)

These live in `app/components/ui/` and are reusable across all pages:

| Component     | Status | Used for                   |
| ------------- | ------ | -------------------------- |
| `AppCard`     | Exists | Journal card container     |
| `AppButton`   | Exists | All buttons                |
| `AppIcon`     | Exists | Edit, delete, create icons |
| `AppChip`     | Exists | Tag display                |
| `AppInput`    | Exists | Form fields, search filter |
| `AppTextarea` | Exists | Description field in form  |
| `AppTagInput` | Exists | Tag editing in form        |
| `AppDialog`   | Exists | Create/edit/delete dialogs |

No new shared components are needed for this page.

### 7.2 Page-Internal Components

These live in `app/components/` (not `ui/`) and are specific to the
Journals page:

| Component                  | Responsibility                                                                                                   |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `JournalCard.vue`          | Single journal card: displays name, description, tags, timestamps. Emits `edit`, `delete`, `click` events.       |
| `JournalForm.vue`          | Exists — form with name, description, tags fields. Used inside the create/edit dialog. Emits `submit`, `cancel`. |
| `JournalDeleteConfirm.vue` | Confirmation dialog content: journal name, warning text, Cancel/Delete buttons. Emits `confirm`, `cancel`.       |

### 7.3 Removed / Replaced

| Component          | Disposition                                                                                         |
| ------------------ | --------------------------------------------------------------------------------------------------- |
| `JournalTable.vue` | **Remove.** A table is the wrong metaphor for 1–5 items with 3 text fields. Replace with card grid. |

## 8. Filter / Search Behavior

A single search input above the card grid filters journals by name and
tags (client-side, since the list is small).

- Filters as the user types (debounced 200ms).
- Matches `Journal.name` (case-insensitive substring) OR any tag
  (case-insensitive substring).
- When filter is active and zero results match, show:
  "No journals match your search."
- The search input is only visible when `journals.length >= 5` (avoid
  visual noise for small lists).

## 9. Responsive Behavior

| Breakpoint          | Grid columns | Card width | Notes                   |
| ------------------- | ------------ | ---------- | ----------------------- |
| < 640px (mobile)    | 1            | Full width | Stack vertically        |
| 640–1024px (tablet) | 2            | ~50%       |                         |
| > 1024px (desktop)  | 3            | ~33%       | Max container width 5xl |

## 10. Page Route and Data Flow

```text
Route:        /                 (app/pages/index.vue)
Layout:       default           (no side menu)
Data:         useJournals()     → JournalClient.list()
Mutations:    JournalClient.create / update / delete → refresh()
```

The existing `useJournals` composable and `JournalClient` interface are
sufficient. No new data layer changes needed.

## 11. Future Considerations

Once journal sub-pages exist (dashboard, accounts, records), the journal
card could show a **summary line** — e.g., account count, record count,
last record date. This requires a backend endpoint that returns aggregated
stats per journal. Not in scope for the initial Journals page.

For the "1 journal" optimization (skip the Journals page and go straight
to the dashboard): this is a UX shortcut that can be added later without
changing the page structure. The Journals page remains the canonical `/`
route regardless.
