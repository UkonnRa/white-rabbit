# Nuxt UI Migration

Replace the custom headless component system with Nuxt UI v4 in a single pass.

Product context: `docs/product/features.md`.
Architecture: `docs/adr/0008-nuxt-ui-migration.md`.

## 1. Scope

**Included:**
- Add `@nuxt/ui` dependency and Nuxt UI module
- Delete all custom UI components (`app/components/ui/*`)
- Delete all theme packages and recipe files (`app/themes/*`)
- Delete all UI-system composables (`useTheme`, `useRecipe`, `useMode`,
  `useRipple`, `useAppTheme`)
- Delete `app/plugins/theme.ts` and `components.json`
- Rewrite `app/assets/css/tailwind.css` as minimal Tailwind v4 entry
- Create `app/app.config.ts` with Nuxt UI theme defaults
- Rewrite all business components to use Nuxt UI equivalents (UButton,
  UInput, UModal, UTable, UChip, UIcon, UCard)
- Rewrite `layouts/default.vue` — remove theme wiring, use Nuxt UI
- Rewrite all pages — replace App* imports with Nuxt UI
- Rewrite endpoint-tauri `app.vue` — remove reka-ui TooltipProvider
- Update tests for all rewritten components
- Remove `@material/material-color-utilities` dependency
- Remove `reka-ui` dependency (Nuxt UI provides it)

**Deferred (not in this migration):**
- No behavioral changes to domain logic or composables
- No changes to client interfaces or Tauri backends
- No new features or pages

## 2. File Inventory

### Files to delete (80+ files)

| Group | Files |
|-------|-------|
| UI components | `app/components/ui/*` (22 files: 15 .vue + 7 .test.ts) |
| Theme packages | `app/themes/*` (47 files: types, registry, md3-expressive, tailwind-default) |
| UI composables | `useTheme.ts`, `useRecipe.ts`, `useMode.ts`, `useRipple.ts`, `useAppTheme.ts` + test files (9 files) |
| Theme plugin | `app/plugins/theme.ts`, `app/plugins/theme.test.ts` (2 files) |
| Demo page | `app/pages/color-demo.vue` |
| Shadcn config | `components.json` |

### Files to create

| File | Purpose |
|------|---------|
| `app/app.config.ts` | Nuxt UI theme configuration |
| `app/assets/css/tailwind.css` | Minimal Tailwind v4 entry (replaces old one) |

### Files to modify

| File | Change |
|------|--------|
| `nuxt.config.ts` | Add `@nuxt/ui` |
| `package.json` | Add `@nuxt/ui`, remove old deps |
| `index.ts` | Remove theme exports if any |
| `layouts/default.vue` | Remove theme composables, use Nuxt UI |
| `layouts/journal.vue` | Use Nuxt UI nav components |
| `pages/index.vue` | App* → U* |
| `pages/journals/[id]/index.vue` | App* → U* |
| `pages/journals/[id]/accounts.vue` | App* → U* |
| `components/JournalCard.vue` | App* → U* |
| `components/JournalForm.vue` | App* → U* |
| `components/AccountTable.vue` | AppDataTable → UTable |
| `components/AccountForm.vue` | App* → U* |
| `components/AccountArchiveConfirm.vue` | AppButton → UButton |
| `components/AccountDeleteConfirm.vue` | App* → U* |
| `components/account-table/*.vue` (3 files) | App* → U* |
| `endpoint-tauri/app.vue` | Remove TooltipProvider |
| `endpoint-tauri/nuxt.config.ts` | Add `@nuxt/ui` |
| `endpoint-tauri/package.json` | Add `@nuxt/ui` |

## 3. Component Mapping

| Old | Nuxt UI | Notes |
|-----|---------|-------|
| AppButton | UButton | variant/size/slot props map directly |
| AppInput | UInput | v-model, placeholder, required |
| AppTextarea | UTextarea | v-model, placeholder |
| AppDialog | UModal | title prop, default slot for body, footer slot |
| AppCard | UCard | default slot for content |
| AppChip | UBadge / UChip | Nuxt UI chip/badge for tags |
| AppIcon | UIcon | name prop ("i-lucide-*" prefix) |
| AppTooltip | UTooltip | text prop, default slot for trigger |
| AppMenu | UDropdownMenu | items array or slot-based |
| AppDataTable | UTable | columns + data props |
| AppToggle | UCheckbox / USwitch | v-model:checked |
| AppSelect | USelect / USelectMenu | v-model, options |
| AppCombobox | UInputMenu | v-model, searchable |
| AppTagInput | UInput with UChip | inline tags |

## 4. Testing Strategy

- Delete all UI-component tests (obsolete, Nuxt UI has its own tests)
- Delete all theme/composable tests (useTheme, useRecipe, etc.)
- Delete integration tests for themes
- Update domain component tests:
  - Change `findComponent({ name: "AppButton" })` to
    `findComponent({ name: "UButton" })`
  - Replace any theme-specific assertions (e.g. checking `--wr-*` classes)
    with Nuxt UI equivalent assertions
  - Verify component rendering, event emission, slot content

## 5. Verification

- `nuxi typecheck` passes in both shared and endpoint-tauri
- `nuxi dev` starts without errors (no socket or import errors)
- `vitest run` passes all tests
- All existing domain functionality preserved
