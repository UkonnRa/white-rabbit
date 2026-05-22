# ADR-0007 (SUPERSEDED): Headless UI System with Token + Recipe Theming

- Status: **Superseded by ADR-0008** (Nuxt UI Migration)
- Date: 2026-04-09
- Superseded: 2026-05-23

> This decision has been reversed. The custom headless component system
> described below has been replaced by Nuxt UI v4. See `ADR-0008-nuxt-ui-migration.md`.
>
> The content below is retained for historical reference only.

## Context (Historical)

The project currently uses Vuetify 4.0.5 as a full-featured Material Design
component library. All three business components (JournalCard, JournalForm,
JournalTable) directly reference Vuetify components (`v-btn`, `v-card`,
`v-data-table`). Theming is handled by `@material/material-color-utilities`
generating MD3 color tokens from a seed color, injected into Vuetify's theme
system.

This works for a single design language but does not meet our longer-term
requirements:

1. **Multi-design-language support.** We need to ship both a clean web-default
   aesthetic and a Material Design 3 Expressive variant, with the option to add
   more design languages later.

2. **Theme vs mode orthogonality.** "Theme" (design language / brand) and
   "mode" (light / dark) must be independent axes. Each theme provides its own
   complete light and dark palettes.

3. **Interaction state abstraction.** Different design languages express
   hover, focus, pressed, and dragged states in fundamentally different ways
   (web: focus ring + scale; MD3: state layer opacity + ripple + elevation).
   Base components must not hardcode any specific interaction implementation.

4. **Token-driven, not palette-only.** Switching theme must affect not just
   colors but also radius, density, spacing, motion, and typography strategies.

5. **No upstream sync burden.** shadcn-vue's copy-paste model means deep
   customisation makes it hard to track upstream changes. We prefer npm
   dependencies for headless primitives so behavioural and a11y improvements
   arrive via normal package upgrades.

6. **Tailwind DX preservation.** Layout utilities (`gap-*`, `p-*`, `w-*`,
   `flex`, `grid`) must remain usable directly in templates. Only component
   internals (variant, size, interaction) are abstracted through the theme
   system.

The project is early (3 business components, SPA-only Tauri target), making
this the right time to establish the UI foundation before more code is written.

## Decision

### 1. Layered architecture

The UI system is organised into five layers. Each layer depends only on the
layers below it.

```text
Layer 4  Domain Components    JournalTable, JournalForm, ...
         (business logic, uses Layer 3 components + Tailwind layout)
  |
Layer 3  Base Components      AppButton, AppInput, AppCard, AppDialog, ...
         (semantic props: variant/size/density; zero hardcoded styles;
          resolves recipe from injected theme)
  |
Layer 2  Theme Packages       themes/tailwind-default/  themes/md3-expressive/
         (each provides: token values + component recipes + interaction recipes)
  |
Layer 1  Theme Infrastructure ThemeProvider composables, recipe resolver,
                              token CSS-variable injection
  |
Layer 0  Headless Primitives  Reka UI (Dialog, Popover, Menu, Tabs, Select, ...)
                              TanStack Table (DataTable)
                              (behaviour + a11y + keyboard nav; zero styling)
```

### 2. Headless primitive selection

**Reka UI** (the v2 rewrite of Radix Vue) is the primary headless library:

- 50+ accessible compound components (Dialog, Popover, Menu, Tabs, Select,
  Combobox, Accordion, Toast, Tooltip, Switch, Checkbox, RadioGroup, Slider).
- `data-state`, `data-disabled`, and other data attributes expose component
  state for CSS targeting.
- `asChild` prop enables polymorphic rendering without wrapper elements.
- Distributed as an npm package; behavioural / a11y fixes arrive via normal
  upgrades.

**@tanstack/vue-table** provides headless data table capabilities (sorting,
filtering, pagination, column visibility, row selection) without any DOM
opinion.

Neither library ships any styles. All visual presentation is our
responsibility.

### 3. Design token architecture

#### 3.1 Two-tier token schema

Tokens are split into a **base tier** (required of all themes) and an
**extension tier** (theme-specific additions).

Base components reference only base-tier tokens. Theme-specific recipes may
reference extension-tier tokens.

```typescript
// themes/types.ts

/** Every theme must satisfy this contract. */
interface BaseTokens {
  color: {
    primary: string;
    onPrimary: string;
    secondary: string;
    onSecondary: string;
    surface: string;
    onSurface: string;
    surfaceVariant: string;
    onSurfaceVariant: string;
    background: string;
    onBackground: string;
    error: string;
    onError: string;
    outline: string;
    outlineVariant: string;
  };
  radius: Record<"sm" | "md" | "lg" | "xl" | "full", string>;
  spacing: Record<"xs" | "sm" | "md" | "lg" | "xl", string>;
  motion: {
    durationFast: string;
    durationNormal: string;
    durationSlow: string;
    easingStandard: string;
    easingEmphasized: string;
  };
}

/** MD3 extends with tertiary palette, state-layer, elevation, etc. */
interface MD3Tokens extends BaseTokens {
  color: BaseTokens["color"] & {
    tertiary: string;
    onTertiary: string;
    tertiaryContainer: string;
    primaryContainer: string;
    onPrimaryContainer: string;
    surfaceContainerLowest: string;
    surfaceContainerLow: string;
    surfaceContainer: string;
    surfaceContainerHigh: string;
    surfaceContainerHighest: string;
    inverseSurface: string;
    inverseOnSurface: string;
    inversePrimary: string;
  };
  stateLayer: {
    hoverOpacity: string;
    focusOpacity: string;
    pressedOpacity: string;
    draggedOpacity: string;
  };
  elevation: Record<
    "level0" | "level1" | "level2" | "level3" | "level4" | "level5",
    string
  >;
}
```

#### 3.2 Token injection: CSS variables + Tailwind v4

Tokens are CSS custom properties scoped by `data-theme` and `data-mode`
attribute selectors:

```css
/* themes/tailwind-default/tokens.css */
[data-theme="tailwind-default"] {
  --wr-primary: oklch(0.55 0.2 260);
  --wr-on-primary: oklch(1 0 0);
  --wr-radius-sm: 0.25rem;
  --wr-radius-md: 0.5rem;
  /* ... */
}
[data-theme="tailwind-default"][data-mode="dark"] {
  --wr-primary: oklch(0.78 0.15 260);
  --wr-on-primary: oklch(0.2 0.08 260);
  /* ... */
}
```

Tailwind v4's `@theme inline` block maps these variables to utility classes:

```css
/* tailwind.css */
@theme inline {
  --color-primary: var(--wr-primary);
  --color-on-primary: var(--wr-on-primary);
  --color-surface: var(--wr-surface);
  --radius-sm: var(--wr-radius-sm);
  --radius-md: var(--wr-radius-md);
  /* ... */
}
```

This makes `bg-primary`, `text-on-primary`, `rounded-md` etc. resolve to the
active theme's token values. Standard layout utilities (`gap-4`, `p-2`,
`w-full`) remain unaffected.

#### 3.3 MD3 dynamic token generation

The existing `@material/material-color-utilities` investment is preserved.
MD3 tokens are generated from a seed colour at runtime via `SchemeTonalSpot`,
then applied as inline CSS custom properties on the scoping element.

### 4. Theme infrastructure: composable-first, no wrapper component

Three composables handle theme lifecycle. No dedicated `<ThemeProvider>`
wrapper component is needed; Vue `provide/inject` scopes context to the
component subtree, and CSS variables scope to a DOM element via bound
data attributes.

```typescript
// composables/useTheme.ts

/**
 * Root initialisation (called once in App.vue or a Nuxt plugin).
 * Sets data-theme / data-mode on <html>, provides recipes to the tree.
 */
function createThemeRoot(options: {
  theme: MaybeRef<string>;
  mode: MaybeRef<"light" | "dark">;
}): { recipes: ComputedRef<ThemeRecipes> };

/**
 * Consumer (called in any component).
 * Returns the current theme's recipes and mode.
 */
function useTheme(): {
  recipes: ComputedRef<ThemeRecipes>;
  mode: ComputedRef<"light" | "dark">;
};

/**
 * Subtree override (rare; returns attrs to bind on an existing element).
 * Overrides provide/inject for descendants and returns data attributes
 * for CSS variable scoping.
 */
function useThemeScope(options: {
  theme: MaybeRef<string>;
  mode: MaybeRef<"light" | "dark">;
}): {
  themeAttrs: ComputedRef<Record<string, string>>;
  recipes: ComputedRef<ThemeRecipes>;
};
```

`createThemeRoot` applies `data-theme` / `data-mode` to `<html>` directly,
avoiding an extra wrapper `<div>` at the root. `useThemeScope` returns
attributes that the caller binds to an already-existing element (a section, a
card, a dialog) for subtree overrides.

This is consistent with how Vuetify 4 structures its own `useTheme()`
composable and avoids the React-style "mandatory Provider wrapper" pattern.

### 5. Recipe system for theme-aware component styling

Each theme provides a **recipe** per component: a structured mapping from
semantic props (variant, size) and interaction states to CSS class lists.

```typescript
// themes/types.ts

interface ComponentRecipe {
  /** Always-applied classes. */
  base: string[];
  /** Variant name -> class list. */
  variants: Record<string, string[]>;
  /** Size name -> class list. */
  sizes: Record<string, string[]>;
  /** Interaction state -> class list (the core abstraction). */
  interactions: {
    hover?: string[];
    focus?: string[];
    pressed?: string[];
    dragged?: string[];
    disabled?: string[];
  };
  /** Optional compound variants for variant+size combinations. */
  compound?: Array<{
    variant?: string;
    size?: string;
    classes: string[];
  }>;
}

interface ThemeRecipes {
  button: ComponentRecipe;
  input: ComponentRecipe;
  card: ComponentRecipe;
  chip: ComponentRecipe;
  dialog: ComponentRecipe;
  menu: ComponentRecipe;
  // ... one entry per base component
}

interface ThemeDefinition {
  name: string;
  recipes: ThemeRecipes;
  /** Optional Vue directives provided by the theme (e.g. ripple). */
  directives?: Record<string, Directive>;
  /** Optional theme-level CSS to import. */
  stylesheets?: string[];
}
```

#### 5.1 Interaction recipe examples

The same base component (`AppButton`) produces entirely different interaction
behaviour depending on the active theme, without any conditional logic in its
own source code:

**tailwind-default** -- focus ring, brightness shift, scale press:

```typescript
interactions: {
  hover:    ['hover:brightness-110'],
  focus:    ['focus-visible:outline-2', 'focus-visible:outline-offset-2',
             'focus-visible:outline-primary'],
  pressed:  ['active:scale-[0.98]'],
  disabled: ['opacity-50', 'pointer-events-none', 'cursor-not-allowed'],
}
```

**md3-expressive** -- state layer via `::before` pseudo-element:

```typescript
interactions: {
  hover:    ['hover:before:opacity-[0.08]'],
  focus:    ['focus-visible:before:opacity-[0.10]'],
  pressed:  ['active:before:opacity-[0.10]'],
  disabled: ['opacity-[0.38]', 'pointer-events-none', 'shadow-none'],
}
```

The MD3 theme additionally registers a `v-ripple` directive for press
feedback, injected via `ThemeDefinition.directives`. Base components are
unaware of its existence.

### 6. Base component pattern

Base components are thin: wrap a Reka UI primitive, inject the recipe, resolve
classes.

```vue
<!-- components/ui/AppButton.vue -->
<script setup lang="ts">
import { Primitive } from "reka-ui";
import { useRecipe } from "../../composables/useRecipe";

const props = withDefaults(
  defineProps<{
    variant?: "solid" | "outlined" | "ghost" | "text";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    as?: string | Component;
  }>(),
  {
    variant: "solid",
    size: "md",
    as: "button",
  },
);

const classes = useRecipe("button", props);
</script>

<template>
  <Primitive :as="as" :class="classes" :disabled v-bind="$attrs">
    <slot />
  </Primitive>
</template>
```

`useRecipe` merges base + variant + size + interaction classes from the
injected theme's recipe. The component itself contains zero style constants.

### 7. File structure

```text
packages/shared/
  app/
    assets/css/
      tailwind.css                  # @theme inline token-to-utility mapping
    components/
      ui/                           # Layer 3: Base Components
        AppButton.vue
        AppInput.vue
        AppTextarea.vue
        AppSelect.vue
        AppCombobox.vue
        AppCard.vue
        AppDialog.vue
        AppChip.vue
        AppDataTable.vue
        AppMenu.vue
        AppTooltip.vue
        AppSwitch.vue
        AppIcon.vue
      domain/                       # Layer 4: Domain Components
        JournalCard.vue
        JournalForm.vue
        JournalTable.vue
    composables/
      useTheme.ts                   # createThemeRoot, useTheme, useThemeScope
      useMode.ts                    # light/dark toggle + persistence
      useRecipe.ts                  # recipe -> class resolution
    themes/                         # Layer 2: Theme Packages
      types.ts                      # BaseTokens, ComponentRecipe, ThemeDefinition
      registry.ts                   # theme name -> ThemeDefinition map
      tailwind-default/
        index.ts                    # ThemeDefinition export
        tokens.css                  # CSS variables (light + dark)
        recipes/
          button.ts
          input.ts
          card.ts
          ...
      md3-expressive/
        index.ts                    # ThemeDefinition export
        tokens.css                  # static fallbacks + MD3 extension tokens
        seed.ts                     # runtime generation via @material/material-color-utilities
        state-layer.css             # ::before pseudo-element + ripple CSS
        ripple.ts                   # v-ripple directive implementation
        recipes/
          button.ts
          input.ts
          card.ts
          ...
    plugins/
      theme.ts                      # Nuxt plugin: createThemeRoot, replaces vuetify.ts
```

### 8. Dependency changes

```text
Added:
  reka-ui                           # headless UI primitives
  @tanstack/vue-table               # headless data table

Removed:
  vuetify                           # replaced by Reka UI + recipe system
  vite-plugin-vuetify               # no longer needed
  @mdi/font                         # replaced by lucide-vue-next (or kept if preferred)

Kept:
  @material/material-color-utilities  # MD3 seed-based token generation
  @vueuse/core                       # utility composables
  tailwindcss + @tailwindcss/vite    # styling foundation
```

### 9. Theme x mode orthogonal model

Theme and mode are independent axes. CSS scoping:

```css
[data-theme="tailwind-default"]                      shared tokens
[data-theme="tailwind-default"][data-mode="light"]   light overrides
[data-theme="tailwind-default"][data-mode="dark"]    dark overrides

[data-theme="md3-expressive"]                        JS-injected from seed
[data-theme="md3-expressive"][data-mode="dark"]      JS-injected from seed (dark)
```

Subtree nesting is supported: a dark MD3 card can live inside a light
web-default page.

## Implementation Phases

Each phase is independently testable and deliverable. A phase is complete when
its acceptance criteria pass. Later phases do not require earlier phases to be
merged -- they can proceed on separate branches -- but the dependency order
must be respected.

### Phase 0: Theme Infrastructure Types and Composables

**Goal:** Establish the TypeScript contracts and composable skeleton that all
subsequent phases build on.

**Deliverables:**

- `themes/types.ts` -- `BaseTokens`, `MD3Tokens`, `ComponentRecipe`,
  `ThemeRecipes`, `ThemeDefinition` type definitions.
- `themes/registry.ts` -- mutable theme registry (name -> ThemeDefinition).
- `composables/useTheme.ts` -- `createThemeRoot`, `useTheme`,
  `useThemeScope` implementations.
- `composables/useRecipe.ts` -- recipe resolver (base + variant + size +
  interaction class merging).
- `composables/useMode.ts` -- light/dark toggle with `useLocalStorage`
  persistence.
- `plugins/theme.ts` -- Nuxt plugin calling `createThemeRoot`.

**Acceptance criteria:**

- [ ] Unit tests for `useRecipe`: given a mock recipe and props, returns the
      correct merged class list.
- [ ] Unit tests for `useTheme` / `useThemeScope`: provide/inject round-trips
      the correct theme definition.
- [ ] Unit tests for `useMode`: toggling mode updates `data-mode` on
      `document.documentElement`.
- [ ] TypeScript compiles with no errors; theme types enforce the base token
      contract.

**No visual output.** This phase is pure infrastructure.

### Phase 1: tailwind-default Theme + First Base Component (AppButton)

**Goal:** Prove the full vertical slice: token CSS -> recipe -> base component
-> rendered output.

**Deliverables:**

- `themes/tailwind-default/tokens.css` -- complete base-tier CSS variables,
  light and dark.
- `themes/tailwind-default/recipes/button.ts` -- button recipe with solid /
  outlined / ghost / text variants, sm / md / lg sizes, hover / focus /
  pressed / disabled interactions.
- `themes/tailwind-default/index.ts` -- ThemeDefinition registering the button
  recipe.
- `tailwind.css` updated with `@theme inline` mapping tokens to Tailwind
  utilities.
- `components/ui/AppButton.vue` -- base button wrapping Reka UI `Primitive`.

**Acceptance criteria:**

- [ ] `AppButton` renders with correct classes for each variant x size
      combination.
- [ ] Switching mode (light/dark) changes token CSS variables and AppButton
      appearance updates.
- [ ] `data-theme="tailwind-default"` and `data-mode` are present on `<html>`.
- [ ] Focus, hover, and disabled states are visually correct (manual or
      snapshot test).
- [ ] No Vuetify imports in AppButton or its dependencies.

### Phase 2: Remaining Base Components (tailwind-default)

**Goal:** Build out the full base component set under the tailwind-default
theme.

**Deliverables:**

- Recipes: `input.ts`, `textarea.ts`, `select.ts`, `combobox.ts`, `card.ts`,
  `chip.ts`, `dialog.ts`, `menu.ts`, `tooltip.ts`, `switch.ts`, `icon.ts`.
- Components: `AppInput`, `AppTextarea`, `AppSelect`, `AppCombobox`,
  `AppCard`, `AppChip`, `AppDialog`, `AppMenu`, `AppTooltip`, `AppSwitch`,
  `AppIcon`.
- `AppDataTable` using `@tanstack/vue-table` with recipe-driven cell /
  header / row styling.

**Acceptance criteria:**

- [ ] Each component renders correctly in light and dark mode.
- [ ] Each component's variant / size / interaction states match the recipe.
- [ ] `AppDataTable` supports sorting, filtering, and inline editing (parity
      with current JournalTable functionality).
- [ ] All components pass basic a11y checks (keyboard navigation, ARIA
      attributes from Reka UI).
- [ ] Storybook-style demo page (or simple `/ui-demo` route) exercises every
      component.

### Phase 3: Migrate Domain Components and Layout

**Goal:** Replace all Vuetify component usage in business code with base
components.

**Deliverables:**

- `domain/JournalCard.vue` rewritten using `AppCard`, `AppButton`, `AppChip`.
- `domain/JournalForm.vue` rewritten using `AppInput`, `AppTextarea`,
  `AppCombobox`, `AppButton`.
- `domain/JournalTable.vue` rewritten using `AppDataTable`, `AppButton`,
  `AppChip`, `AppCombobox`, `AppInput`.
- `layouts/default.vue` rewritten: custom app bar, navigation drawer, footer
  using base components + Tailwind layout.
- Remove `vuetify`, `vite-plugin-vuetify`, `@mdi/font` from dependencies.
- Remove `plugins/vuetify.ts` and the Vuetify CSS layer declarations from
  `tailwind.css`.

**Acceptance criteria:**

- [ ] All existing application functionality works identically (create, edit,
      delete journals; inline table editing; tag management; theme colour
      switching; dark mode toggle).
- [ ] Zero Vuetify imports remain in the codebase.
- [ ] Bundle size does not regress beyond the size of added dependencies.
- [ ] Navigation, drawer toggle, footer all function correctly.

### Phase 4: md3-expressive Theme

**Goal:** Prove multi-design-language support by implementing a complete
second theme.

**Deliverables:**

- `themes/md3-expressive/seed.ts` -- `generateMD3Tokens(seedHex, dark)`
  using `@material/material-color-utilities`.
- `themes/md3-expressive/tokens.css` -- MD3 extension token declarations.
- `themes/md3-expressive/state-layer.css` -- `::before` pseudo-element rules
  for state layer feedback.
- `themes/md3-expressive/ripple.ts` -- `v-ripple` Vue directive.
- `themes/md3-expressive/recipes/` -- button, input, card, chip, dialog,
  menu, data-table recipes using full-round corners, state layer interactions,
  MD3 elevation, MD3 typography scale.
- `themes/md3-expressive/index.ts` -- ThemeDefinition with directive
  registration.
- Theme switcher UI in the app bar (dropdown or toggle between
  tailwind-default and md3-expressive).

**Acceptance criteria:**

- [ ] Switching from tailwind-default to md3-expressive changes: corner radius
      strategy, interaction feedback (state layer vs ring), elevation vs flat,
      typography scale, colour palette.
- [ ] MD3 seed colour picker works: changing the seed colour regenerates
      tokens for both light and dark modes.
- [ ] Ripple directive fires on press for MD3 buttons.
- [ ] All domain components render correctly under both themes.
- [ ] Subtree theme override works: a md3-expressive dialog can appear inside
      a tailwind-default page.

### Phase 5: Polish and Hardening

**Goal:** Production-readiness pass.

**Deliverables:**

- Transition / animation tokens wired up and tested across themes.
- Density variants (compact / comfortable / default) if needed.
- Accessibility audit: screen reader testing, focus order, colour contrast
  under both themes x both modes.
- Performance audit: bundle analysis, CSS size, runtime repaint cost of
  theme switching.
- Documentation: update architecture overview, component catalogue page.

**Acceptance criteria:**

- [ ] axe-core or similar automated a11y scan passes with zero critical
      issues.
- [ ] Lighthouse accessibility score >= 95.
- [ ] Theme switch (including MD3 seed colour change) completes within 100ms
      with no visible flash.
- [ ] No unused CSS variables or dead recipe code.

## Consequences

### Positive

- Base components are styling-agnostic: adding a new theme requires only a
  new `themes/*/` directory with token CSS and recipe files, zero changes to
  component source.
- Interaction states are a first-class design dimension, not an afterthought
  hardcoded into components.
- Theme and mode are orthogonal: any combination works, including subtree
  overrides.
- Headless primitives (Reka UI, TanStack Table) receive a11y and behaviour
  updates via normal npm upgrades, no copy-paste sync burden.
- Tailwind DX is preserved for layout; only component variant/interaction
  styling goes through the recipe system.
- The MD3 seed-colour investment (`@material/material-color-utilities`) is
  retained and scoped to the md3-expressive theme.

### Negative

- No off-the-shelf component library: every base component must be built and
  maintained in-house, including edge cases (loading states, error states,
  complex form validation display).
- Recipe system is a custom abstraction: new contributors must learn it.
  Mitigated by keeping the recipe type interface small and well-documented.
- Two themes means double the recipe surface to maintain. Each new base
  component requires a recipe in every theme.
- TanStack Table has a steeper learning curve than Vuetify's `v-data-table`
  for common CRUD table patterns.
- Removing Vuetify removes its layout system (`v-app`, `v-main`,
  `v-container`); layout must be rebuilt with Tailwind utilities.

## Alternatives Considered

### 1. Keep Vuetify 4 + add a secondary non-Vuetify theme

Use Vuetify as-is for MD3 and build a parallel web-default theme on a
different primitive. Rejected: the base component API would differ between
themes (Vuetify props vs custom props), making domain components theme-aware.

### 2. shadcn-vue as the component layer

Adopt shadcn-vue components directly and customise. Rejected: copy-paste
model means component source becomes ours to maintain and upstream sync is
difficult. Additionally, shadcn-vue bakes specific Tailwind classes (ring
widths, specific rounded values, hardcoded spacing) into component source,
which conflicts with multi-design-language support.

### 3. PrimeVue unstyled + pass-through API

Use PrimeVue in unstyled mode with PT API for styling injection. Rejected:
PT API couples styling to PrimeVue's internal DOM structure (section names
like `root`, `label`, `icon`). Switching presets changes colours but not
interaction patterns, density strategies, or corner radius philosophies.

### 4. Vuetify0

Use Vuetify0 as the headless primitive layer. Rejected: too new, small
component set, insufficient community validation. However, its composable
patterns (`createTokens`, `createContext`, `useTheme` subtree scoping) are
architecturally influential and partially adopted in our theme infrastructure
design.

### 5. CSS-in-JS or style-dictionary approach instead of Tailwind classes

Define recipes as CSS objects or use a tool like style-dictionary. Rejected:
loses Tailwind's utility-class DX, adds a build step, and requires a
different mental model from the rest of the codebase.

## References

- Reka UI (headless Vue primitives): <https://reka-ui.com/>
- Reka UI styling guide: <https://reka-ui.com/docs/guides/styling>
- Reka UI composition guide: <https://reka-ui.com/docs/guides/composition>
- TanStack Table: <https://tanstack.com/table/>
- Tailwind CSS v4 `@theme` directive: <https://tailwindcss.com/docs/theme>
- Material Design 3 interaction states: <https://m3.material.io/foundations/interaction/states>
- Material color utilities: <https://github.com/nicolo-ribaudo/material-color-utilities>
- shadcn-vue theming: <https://shadcn-vue.com/docs/theming>
- Vuetify0 architecture: <https://0.vuetifyjs.com/llms.txt>
- ADR-0006 (Nuxt layers / client abstraction): `0006-nuxt-frontend-client-abstraction.md`
