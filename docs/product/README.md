# Product Overview

White Rabbit is a personal and family finance tracker inspired by
[beancount](https://beancount.github.io/docs/) and
[fava](https://github.com/beancount/fava).
It brings double-entry bookkeeping to everyday household use with a
desktop-first experience and an optional self-hosted web mode.

## Core idea

Double-entry bookkeeping for home use.
Every financial move is recorded as a _transaction_ that balances across two
or more accounts, giving an always-consistent view of where money is and
where it came from.

## Deployment modes

| Mode       | Host                   | Auth               | Persistence                |
| ---------- | ---------------------- | ------------------ | -------------------------- |
| **Local**  | Tauri desktop app      | None — single-user | Encrypted SQLite on device |
| **Remote** | Self-hosted web server | Full AuthN + AuthZ | Server-side database       |

The local mode is the primary target.
No family member wants their home finance data to leave their device unless
they explicitly choose to self-host.

In local mode, the auth layer **does not exist** — `crates/domain` has no
knowledge of users or permissions.
In remote mode, auth is layered on top by the endpoint, never baked into the
domain.

## Relation to beancount

| Concept       | beancount                 | White Rabbit                    |
| ------------- | ------------------------- | ------------------------------- |
| File / ledger | `.beancount` text file    | `Journal`                       |
| Account tree  | `Assets:Bank:Checking`    | `Account` (tree, typed)         |
| Transaction   | `YYYY-MM-DD * "desc" ...` | `Record` + `RecordItem`s        |
| Price / cost  | `@ / {}` syntax           | `Amount.price` / `Amount.cost`  |
| Assertion     | `balance` directive       | `RecordItemValidation`          |
| Report UI     | fava                      | White Rabbit UI (fava-inspired) |

## References

- beancount language: <https://beancount.github.io/docs/beancount_language_syntax.html>
- fava UI: <https://github.com/beancount/fava>
- beancount design doc: <https://docs.google.com/document/d/1RaondTJCS_IUPBHFNdT8oqFKJjVJDsfsn6JEjBG04eA/>
