# ADR-0008: Migrate to Nuxt UI

- Status: Accepted
- Date: 2026-05-23

## Context

The project previously implemented a custom headless component system
(ADR-0007) with Reka UI primitives, a recipe-driven theming layer,
and dual design-language support (tailwind-default + md3-expressive).
This system required maintaining 14 custom base components, 28 recipe
files, and 5 theme composables — all for a single SPA Tauri app with
3 business components.

## Decision

Replace the entire custom UI system with **Nuxt UI v4** as the sole
component library.

Nuxt UI v4 is built on Reka UI and TanStack, provides 125+ pre-built
components, and integrates natively with Nuxt 4 + Tailwind CSS v4.
It offers a single, cohesive design language with theming via
`app.config.ts`. All custom components (AppButton, AppInput, AppDataTable,
etc.), theme packages, recipe files, and CSS token variables (`--wr-*`)
are removed. Business components are rewritten to use Nuxt UI equivalents.

## Consequences

### Removed

- All `app/components/ui/` components (AppButton, AppInput, AppDialog,
  AppDataTable, AppChip, AppIcon, AppTextarea, AppTagInput, AppCard,
  AppTooltip, AppMenu, AppBadge, AppToggle)
- Theme packages (`tailwind-default/`, `md3-expressive/`) and all recipe files
- Theme composables (`useTheme`, `useRecipe`, `useMode`, `useRipple`,
  `useAppTheme`)
- `@material/material-color-utilities` dependency
- `--wr-*` CSS custom properties and token files
- `v-ripple` directive

### Added

- `@nuxt/ui` dependency
- Nuxt UI module in `nuxt.config.ts`
- `app.config.ts` with Nuxt UI theme configuration

### Changed

- All business components (`JournalCard`, `JournalForm`, `JournalTable`,
  `AccountTable`, `AccountForm`, `AccountArchiveConfirm`,
  `AccountDeleteConfirm`) migrate from App* components to Nuxt UI
  components (UButton, UInput, UModal, UTable, etc.)
- `AppDataTable` is replaced by Nuxt UI's built-in table
- `layouts/journal.vue` sidebar uses Nuxt UI navigation components
- Theme toggle (if any) simplifies to light/dark mode

### Preserved

- `@tanstack/vue-table` (peer dependency of Nuxt UI)
- `@iconify/vue` (Nuxt UI uses Iconify)
- Domain models, composables, and client interfaces are unchanged

## Migration completeness

This migration is a complete, one-time replacement. No dual-track or
gradual migration period. The custom system is removed in a single pass
and all references are updated to Nuxt UI.
