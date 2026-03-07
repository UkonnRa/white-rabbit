# ADR-0006: Nuxt 4 Layers with Frontend Client Abstraction

- Status: Accepted
- Date: 2026-03-02

## Context

The project needs a shared frontend (components, pages, composables) that works
across a Tauri desktop app and a future web endpoint. Data comes from different
sources (Tauri `invoke` vs HTTP API), but pages should be source-agnostic.

The prior setup used plain Vue 3 with `provide/inject` for dependency injection
of a `JournalClient` interface. This worked but has limitations in Nuxt:

1. Nuxt has no `main.ts` entry point, so `app.provide()` cannot be called
   directly; a plugin is required either way.
2. Vue `inject()` only works inside component `setup()`. Nuxt composables are
   also used in middleware, error handlers, and other contexts outside the
   component tree.
3. Nuxt plugin `provide` gets automatic TypeScript augmentation via
   `declare module '#app'`, while Vue's `inject()` requires manual typing.

## Decision

### 1. Nuxt 4 layers for shared UI

`@white-rabbit/shared` is a Nuxt layer that provides components, composables,
and pages. Endpoint apps (`endpoint-tauri`, future `endpoint-web`) extend it
via `nuxt.config.ts`:

```ts
extends: ['@white-rabbit/shared']
```

### 2. One client interface, multiple implementations

A single `JournalClient` interface with `Promise<T>` returns is defined in the
shared package. Each endpoint provides its implementation:

- **Tauri**: `TauriJournalClient` uses `invoke()` from `@tauri-apps/api`.
- **Web** (future): an HTTP-based client using `$fetch`.

The client instance is injected via a Nuxt plugin using `defineNuxtPlugin` +
`provide`, and resolved by shared composables via `useNuxtApp().$journalClient`.

### 3. Composable abstraction with `useAsyncData`

Shared composables wrap client reads in `useAsyncData`, returning `AsyncData<T>`:

- `useJournals(filter?)` returns `AsyncData<Journal[]>`
- `useJournal(id)` returns `AsyncData<Journal>`

Endpoint apps can override these composables entirely. For example, a web
endpoint could replace `useJournals` with a `useFetch('/api/journals')` call.
Since both `useAsyncData` and `useFetch` return `AsyncData<T>`, shared pages
work with either backend unchanged.

### 4. Reads vs writes

- **Reads**: shared composables using `useAsyncData(() => client.method())`.
- **Writes**: `useJournalClient()` returns the injected client for imperative
  mutation calls (`create`, `update`, `delete`), followed by `refresh()`.

### 5. Tauri runs Nuxt in SPA mode

Tauri endpoint sets `ssr: false` in `nuxt.config.ts`. `nuxi generate` outputs
static files to `.output/public/` for Tauri's webview.

## Consequences

### Positive

- Shared UI code works across all endpoints without modification.
- Adding a new endpoint only requires a `nuxt.config.ts` extending the layer
  and a plugin providing the client implementation.
- Composable override gives full flexibility for endpoint-specific data
  fetching patterns.
- TypeScript augmentation makes `$journalClient` type-safe across the app.

### Negative

- Nuxt dependency is added to the shared package, increasing its footprint.
- Composable auto-import requires understanding Nuxt layer resolution order
  when overriding in endpoint apps.
- SPA mode loses SSR benefits for the Tauri endpoint (acceptable since it
  runs in a local webview).

## Alternatives Considered

1. **Vue `provide/inject`** — Works inside component `setup()` but not in
   middleware or error handlers. A Nuxt plugin is needed either way since
   there is no `main.ts`.

2. **`app.config` / `useAppConfig()`** — Designed for serializable reactive
   configuration data, cannot hold class instances with methods.

3. **Nuxt module instead of layer** — Too heavy for UI sharing; modules are
   for build-time extensions, not runtime UI composition.

4. **Abstract `useFetch` wrapper hiding source** — Leaky abstraction across
   Tauri `invoke` and HTTP; `invoke` does not use HTTP semantics.

## References

- Nuxt Layers: https://nuxt.com/docs/4.x/guide/going-further/layers
- Nuxt Plugins (providing helpers): https://nuxt.com/docs/4.x/directory-structure/app/plugins
- Nuxt Composables (plugin injection access): https://nuxt.com/docs/4.x/directory-structure/app/composables
- Architecture Overview, section 14
