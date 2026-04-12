# Architecture Overview

This document defines the project architecture in a language-neutral way.
It explains principles, runtime behavior, and boundaries without tying
decisions to any specific programming language or framework.

## 1. Goals

- Use DDD aggregate boundaries as the source of business invariants.
- Reuse the same command logic for both real execution and dry-run
  simulation.
- Support CQRS in practical systems where write-side decisions and
  complex queries can coexist.
- Use Specification as a shared query intent model across read and write
  sides.

## 2. Non-Goals

- Rebuilding a full in-memory SQL join engine or datastore query planner.
- Forcing an immediate redesign of all existing command flows to avoid
  complex command-time queries.

## 3. Core Principles

### 3.1 Unit of Work Is About More Than Transaction Rollback

Unit of Work is used to:

- Track change sets and snapshots.
- Build accurate dry-run diffs.
- Stage side effects for controlled dispatch.
- Provide a unified boundary for internal event processing.

### 3.2 Domain Events Should Be Business-Semantic

Use business-meaningful events, not generic CRUD events.

- Good: `ShippingInfoRefreshed`
- Weak: `EntityUpdated`

Business-semantic events improve readability, handler design, and dry-run
side-effect control.

### 3.3 Command-Time Queries Must Be Write-Consistent

Command decisions must query write-consistent data sources.
Read models can be eventually consistent and are not authoritative for
write-time decisions.

## 4. Runtime Modes

### 4.1 Real Run

- Persist writes to the primary write store.
- Commit transaction.
- Dispatch external side effects after commit (or via outbox).

### 4.2 Dry Run

- Do not mutate the primary write store.
- Execute internal deterministic/in-transaction reactions.
- Capture external side effects without executing them.
- Return structured diff and side-effect preview.

## 5. Components And Responsibilities

- **Command Bus**: command execution entrypoint and execution context
  setup.
- **Execution Context**: request/transaction scoped runtime mode and
  unit-of-work reference.
- **Unit of Work**: identity map, snapshots, event buffers, and diff
  builder.
- **Repository**: stateless adapter that attaches loaded entities to
  current unit-of-work.
- **Event Router**:
  - Internal/in-process events (allowed in dry-run).
  - External side-effect events (blocked in dry-run, captured only).
- **Query Engine**: executes Specifications against write store and
  dry-run overlays.
- **Dry-Run Result Builder**: entity/relation diffs, executed internal
  events, intercepted side effects.

## 5.1 Aggregate And Command Handler Contract

- Command handler (`handle`) validates business rules and returns produced
  events directly.
- The aggregate does not buffer or publish events itself.
- Application service receives events from `handle`, calls `apply` on each
  to update aggregate state, then passes events to Unit of Work for
  buffering.
- `apply` is a pure function: deterministic, no validation, no
  non-determinism (clocks, random IDs), no rejection of historical events.
  All non-deterministic values are fixed at command time and captured in
  event fields.
- Unit of Work holds the event buffer and orchestrates persistence and
  dispatch.
- The full flow is: `handle()` produces events -> application service calls
  `apply()` per event -> Unit of Work buffers events and snapshot -> Unit of
  Work persists and dispatches.

## 6. Event Processing Model

### 6.1 Phase A: Internal In-Transaction Events

- Executed until stable (no new internal events).
- Must be deterministic and free of irreversible external IO.
- All Phase A handlers share the command's database transaction.
- Handlers execute synchronously (no concurrency within Phase A).
- The event pattern is used for local service decoupling; publishing is
  effectively a synchronous in-process method call within the transaction
  boundary.
- Any handler error rolls back the entire transaction, including the
  originating command's writes.

### 6.2 Phase B: External Side Effects

- Real run: dispatch via transactional outbox. Outbox records are written
  inside the command transaction; a background worker delivers them
  asynchronously after commit. This guarantees at-least-once delivery
  without coupling commit success to external system availability.
- Dry run: do not dispatch; record planned side effects.

### 6.3 Persistence Model

- The system uses Event + Snapshot dual-write.
- Both the event stream and the current aggregate snapshot are persisted in
  the same database transaction.
- Events provide audit trail and time-travel capability (replay to any past
  state).
- Snapshots provide efficient read-after-write and query access on the
  write side.
- See `docs/adr/0002-event-sourcing-dual-write-persistence.md`.

## 7. Dry-Run Data Isolation

Use a virtual overlay model:

- **Primary**: canonical write store.
- **Staging Overlay**: transaction-scoped delta records for dry-run.
- Query resolution precedence: `overlay > primary`, with delete tombstone
  handling.

Only changed entities are staged. Unchanged data is read from primary.

## 8. Specification Model

- Specification defines query intent only.
- Executors/adapters translate intent into storage-specific queries.
- Read and write sides can share Specification semantics while using
  different executors.

## 9. Error Design Boundary

- Shared package errors should remain generic and reusable.
- Domain/application errors should carry context required for diagnostics
  and API responses.
- Context enrichment happens at domain/application boundaries.

## 10. Transaction Boundaries And Long-Running Processes

- Database transactions must be short: contain only DB-intensive operations.
- External I/O (HTTP calls, message broker sends, third-party APIs) must
  not occur inside a database transaction.
- Long-running business flows that span external waits must be modeled as
  explicit state machines with persisted intermediate states.
- Recovery from crashes or partial failures uses background sweepers that
  detect and compensate zombie intermediate states.
- Cross-service multi-step flows use Saga with compensation.
- See `docs/adr/0004-transaction-boundaries-long-running-processes.md`.

## 11. Event Bus Routing

- In-process event bus uses topic-based routing with guaranteed delivery
  (no data loss on slow consumers).
- Subscription granularity is per domain-event category (one topic per
  aggregate or bounded context), not per individual event type.
- Subscribers filter for specific event variants locally after receiving
  category-level messages.
- See `docs/adr/0003-in-memory-event-bus-routing.md` for
  implementation-specific choices.

## 12. Unit of Work Implementation

- UoW is a generic, type-erased in-memory change tracker (new/dirty/deleted
  entities + event buffer).
- UoW overlay queries merge in-memory changes with database results:
  tombstones removed, new/dirty merged.
- Write-path UoW queries must be single-aggregate-type; cross-aggregate
  resolution is decomposed at the service level.
- `SpecificationEvaluator` provides in-memory specification matching for
  overlay queries.
- `WriteService` has two methods: `do_handle` (pure, side-effect-free) and
  `handle` (orchestrates persistence).
- See `docs/adr/0005-unit-of-work-write-service-split.md`.

## 13. Frontend Architecture

### 13.1 Nuxt Layer Model

The frontend is structured as a Nuxt 4 layer
(`@white-rabbit/shared`) extended by endpoint apps.
The shared layer provides components, composables, and pages.
Endpoint apps (e.g., `endpoint-tauri`, future `endpoint-web`) extend it via
`nuxt.config.ts` and supply their own client implementation.

Directory convention for the shared layer:

- `app/components/` — auto-registered Vue components.
- `app/composables/` — auto-imported composables.
- `app/pages/` — file-based routing pages.
- `models/` — domain types, outside `app/` for package-level exports.
- `clients/` — client interface, outside `app/` for package-level exports.

### 13.2 Client Abstraction

One `JournalClient` interface with `Promise<T>` returns.
Each endpoint provides its implementation via a Nuxt plugin
(`defineNuxtPlugin` + `provide`).
Shared composables resolve the client via
`useNuxtApp().$journalClient`.

The client interface lives in the shared package so that both the shared
composables and endpoint implementations can reference it.
Endpoint implementations are not in the shared package; they live in each
endpoint's own `clients/` directory.

### 13.3 Composable Pattern

Shared composables wrap client reads in
`useAsyncData(() => client.method())`, returning `AsyncData<T>`.
Extending apps can override composables (e.g., replace with `useFetch` for
a web endpoint).
Mutations use the client directly via `useJournalClient()`, calling
`refresh()` after writes.

### 13.4 Rendering Modes

- **Tauri** (`endpoint-tauri`): `ssr: false` (SPA). `nuxi generate`
  outputs static files for the webview.
- **Web** (future): may use SSR or SSG depending on requirements.

See `docs/adr/0006-nuxt-frontend-client-abstraction.md`.

## 14. Product Context

For the product vision, use-case definitions, feature rules, and auth
isolation strategy, see `docs/product/`:

- `docs/product/README.md` — project overview and relation to beancount
- `docs/product/features.md` — Journal, Account, Record, auth, reporting,
  package responsibilities
- `docs/product/auth-isolation.md` — open question: how to enforce auth
  without touching the domain

## 15. Decision Records

Architecture decisions and trade-offs are recorded in ADRs:

- See `docs/adr/README.md`
- See `docs/adr/0001-ddd-dry-run-cqrs-specification.md`
- See `docs/adr/0002-event-sourcing-dual-write-persistence.md`
- See `docs/adr/0003-in-memory-event-bus-routing.md`
- See `docs/adr/0004-transaction-boundaries-long-running-processes.md`
- See `docs/adr/0005-unit-of-work-write-service-split.md`
- See `docs/adr/0006-nuxt-frontend-client-abstraction.md`
- See `docs/adr/0007-headless-ui-system-with-token-recipe-theming.md`

## 16. Headless Component Library — Positioning

### 16.1 Core Idea

The component library separates headless behaviour (Reka UI, TanStack Table)
from a recipe-driven styling layer, so that the same set of business
components can render under structurally different design languages — not
just different color values, but different interaction models, surface
strategies, and color derivation algorithms — without conditional logic in
component source code.

### 16.2 Problem: Design-Language Lock-In

Mainstream component libraries couple behaviour and styling:

| Approach                | Examples                       | What you can swap                         | What you cannot swap                                                                                     |
| ----------------------- | ------------------------------ | ----------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Full-suite library      | Vuetify, Ant Design, PrimeVue  | Colors, radii, density via tokens/presets | Interaction paradigm (state layer vs focus ring), surface hierarchy strategy, color derivation structure |
| Copy-paste headless     | shadcn/ui                      | Theoretically everything                  | In practice, changes scatter across every copied file; upstream a11y fixes cannot be synced              |
| Unstyled + pass-through | PrimeVue unstyled, Headless UI | Class names on predefined sections        | PT API couples styling to internal DOM section names; cannot change interaction model                    |

All three lock the consumer into one design-language philosophy once
non-trivial customisation begins.

### 16.3 Our Approach: Recipe-Separated Theming

```text
Business components (JournalTable, JournalForm)
        ↓ uses
Base components (AppButton, AppInput, AppCard, …)
        ↓ injects recipe from
Theme packages (tailwind-default/ , md3-expressive/)
        ↓ wraps
Headless primitives (Reka UI, TanStack Table)   ← npm upgrades
```

Each theme package provides:

- **Token CSS** — `--wr-*` custom properties scoped by `data-theme` ×
  `data-mode`, with a per-theme seed-to-token derivation algorithm.
- **Component recipes** — `ComponentRecipe` objects mapping
  variant/size/interaction props to Tailwind class lists.
- **Directives** (optional) — e.g. MD3's `v-ripple`, applied by the
  `useRipple` composable only when the active theme declares it.

Base components contain zero style constants. `useRecipe("button", props)`
resolves classes from the injected theme; `useRipple(elRef)` attaches or
detaches interaction effects as the theme changes.

### 16.4 Structural Color Gap

The two themes differ not just in palette values but in how colors are
derived from a seed:

| Dimension            | md3-expressive                                        | tailwind-default                                                             |
| -------------------- | ----------------------------------------------------- | ---------------------------------------------------------------------------- |
| Derivation           | Single seed → all 29+ roles via HCT `SchemeTonalSpot` | Seed extracts hue → oklch accent scale; surfaces stay on fixed neutral scale |
| Secondary / Tertiary | Algorithmically derived (hue-shifted)                 | Neutral (no colored secondary); no tertiary concept                          |
| Surfaces             | 5-level container hierarchy, tinted toward primary    | 2-level surface / surface-variant from neutral ramp, no primary tint         |
| Dark mode            | Tonal palette tone flip (T80↔T20)                     | Scale index flip (shade-600↔shade-400, neutral-50↔neutral-900)               |

Changing the seed in MD3 shifts the entire mood (surfaces, secondary,
containers all move). Changing the seed in tailwind-default shifts only the
accent — surfaces remain clean neutral.

See `docs/architecture/color-palette-design.md` for the full comparison
with Radix Colors, Ant Design, PrimeVue, and shadcn/ui.

### 16.5 Strengths

- **Interaction states are a first-class design dimension.** Hover, focus,
  pressed, disabled are declared per-theme in recipes, not hardcoded in
  components. MD3 uses state layer + ripple; Tailwind uses focus ring +
  scale — `AppButton.vue` has zero conditional branches.
- **Color derivation is structurally different per theme**, not just
  different values in the same slots. This produces a genuine visual gap
  between design languages.
- **Headless primitives arrive via npm.** Reka UI and TanStack Table
  a11y/keyboard fixes propagate through normal package upgrades, unlike
  shadcn's copy-paste model.
- **Dark mode is orthogonal to theme and seed color.** Three independent
  axes (design language × seed color × light/dark) compose freely.

### 16.6 Weaknesses

- **N×M recipe maintenance.** Every base component needs a recipe in every
  theme. Currently 14 components × 2 themes = 28 recipe files. Adding a
  third theme adds 14 more.
- **Custom abstraction.** The recipe system is project-specific; new
  contributors must learn it. No community ecosystem to draw from.
- **Edge-case coverage is self-built.** No equivalent of Vuetify's
  feature-dense `v-data-table` (virtual scroll, server-side pagination,
  grouped rows). Complex patterns must be implemented from scratch.
- **Runtime JS cost for theme switch.** Seed token regeneration runs JS
  (`el.style.setProperty` in a loop) rather than pure CSS selector toggle.
  Measured under 100ms, but strictly more expensive than a class swap.
