# Feature Specification

## 1. Journal

A Journal is the top-level container — the equivalent of a beancount ledger file.
All accounts, records, and reports belong to exactly one Journal.

### Rules

- Journals are fully isolated: data in one Journal is invisible to another.
- Two Journals cannot be merged.
- Each Journal has its own account tree, starting with the 5 root accounts
  (Asset, Liability, Equity, Income, Expense).

### Operations

- Create, rename, update description / tags, delete.
- In remote mode: manage member roles (see §4).

## 2. Account

Accounts form a tree, mirroring beancount's colon-separated hierarchy
(`Assets:Bank:Checking` → three nested accounts).

### Rules

- Each account inherits its type from its parent.
  The 5 root accounts are created by the system when a Journal is created;
  they cannot be deleted or renamed by users.
- Account names must be unique among siblings.
- Reserved root names (Asset, Liability, Equity, Income, Expense) may not be
  used for non-root accounts.
- Accounts can be archived; archived accounts cascade to all descendants.
  Archived accounts cannot receive new records.
- Account type is immutable after creation (tree structure is fixed).

### Operations

- Create (under an existing parent), rename, update description / tags,
  archive, delete (cascades to descendants).
- Batch operations for bulk import.

### Planned: currency / commodity annotation

Each account will be annotatable with one or more currency or commodity
symbols (e.g. `USD`, `EUR`, `AAPL`).
This is not yet implemented in the domain.

## 3. Record and RecordItem

A Record is a single financial event — the equivalent of a beancount
transaction or balance assertion.

### Record fields

- `date` — the effective date of the event.
- `description` — free-text narrative.
- `payee` — optional counterparty.
- `tags` — free-form labels for filtering and reporting.
- `items` — either a set of Transactions or a set of Validations (not both).

### RecordItem — Transaction

Maps to beancount's posting line:

```text
2024-03-01 * "Groceries"
  Expenses:Food          42.50 USD
  Assets:Bank:Checking  -42.50 USD
```

Fields: `account_id`, `amount` (value + unit/currency), optional `price`
(`@` in beancount), optional `cost` (`{}` in beancount), `description`.

The record is balanced when the sum of all postings is zero (standard
double-entry rule). Validation is enforced at save time.

### RecordItem — Validation

Maps to beancount's `balance` directive — asserts the current balance of an
account at a given date.

### Operations

- Create, update, delete.
- Batch operations.

## 4. Authorization (remote mode only)

Authorization is applied at the Journal level.
Each Journal has three role tiers:

| Role       | Can do                                                                           |
| ---------- | -------------------------------------------------------------------------------- |
| **Admin**  | Full control: edit Journal metadata, manage members, edit Accounts, edit Records |
| **Member** | Edit Accounts and Records; cannot edit or delete the Journal itself              |
| **Reader** | Read-only access; suitable for sharing reports                                   |

### Rules

- In local (Tauri) mode, auth does not exist. The domain layer has no
  knowledge of users or roles.
- In remote mode, auth is enforced by the endpoint layer, not by
  `crates/domain`. The domain processes commands without checking identity.
- `crates/shared-auth` provides the primitives (`AuthEntity`, `Permission`,
  `AuthReadService`) used only by remote endpoints.
- `crates/domain` must never import `crates/shared-auth`.

### Open question: how does the remote endpoint attach auth context to

commands without polluting domain types?
See `docs/product/auth-isolation.md`.

## 5. Reporting (UI — planned)

Inspired by fava. Target reports:

| Report                  | Description                                      |
| ----------------------- | ------------------------------------------------ |
| **Balance sheet**       | Assets − Liabilities = Equity at a point in time |
| **Income statement**    | Income vs Expenses over a period                 |
| **Cash flow**           | Net movement through asset accounts              |
| **Account register**    | All records for a given account, paginated       |
| **Journal overview**    | All records across the whole Journal             |
| **Holdings**            | Commodity/stock positions with cost basis        |
| **Net worth over time** | Time-series chart                                |

## 6. Package responsibilities

| Package                            | Responsibility                                                                                                                               |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/shared`                    | Generic reusable primitives (not White Rabbit specific): `Id`, `NonEmpty`, `Entity`, `Specification`, `UnitOfWork`, error kinds              |
| `crates/shared-auth`               | Auth primitives (`AuthEntity`, `Permission`): used only by remote endpoints, never imported by `domain`                                      |
| `crates/domain`                    | White Rabbit domain logic: `Journal`, `Account`, `Record` aggregates, commands, events, services, specifications. No HTTP, no Tauri, no auth |
| `crates/domain-database-*`         | Storage adapters for domain repositories                                                                                                     |
| `packages/endpoint-tauri`          | Thin Tauri command wrapper. Local only, no auth. Exposes domain services as Tauri IPC commands                                               |
| `packages/endpoint-web` _(future)_ | HTTP endpoint with full auth, SSR/SSG, multi-user                                                                                            |
