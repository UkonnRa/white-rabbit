import type { Account, AccountRow } from "../models";
import { AccountType } from "../models";

export function useAccounts(journalId: MaybeRef<string>) {
  const client = useAccountClient();
  return useAsyncData(
    `accounts:${toValue(journalId)}`,
    () => client.list({ journalId: toValue(journalId) }),
    { watch: isRef(journalId) ? [journalId] : undefined },
  );
}

export function useAccount(id: MaybeRef<string>) {
  const client = useAccountClient();
  return useAsyncData(`account:${toValue(id)}`, () => client.get(toValue(id)), {
    watch: isRef(id) ? [id] : undefined,
  });
}

export function buildAccountRows(accounts: Account[]): AccountRow[] {
  const TYPE_ORDER = [
    AccountType.Asset,
    AccountType.Liability,
    AccountType.Equity,
    AccountType.Income,
    AccountType.Expense,
  ];
  const byParentId = new Map<string | null, Account[]>();

  for (const a of accounts) {
    const key = a.parentId ?? null;
    if (!byParentId.has(key)) byParentId.set(key, []);
    byParentId.get(key)!.push(a);
  }

  function buildRow(account: Account, depth: number): AccountRow {
    const row: AccountRow = { ...account, subRows: [], depth };
    const children = byParentId.get(account.id) ?? [];
    row.subRows = children.map((c) => buildRow(c, depth + 1));
    return row;
  }

  const roots: AccountRow[] = [];
  for (const type of TYPE_ORDER) {
    const typeRoots =
      byParentId.get(null)?.filter((a) => a.type === type) ?? [];
    for (const a of typeRoots) {
      roots.push(buildRow(a, 0));
    }
  }

  return roots;
}

export function useAccountTree(
  accounts: Ref<Account[]>,
  showArchived: Ref<boolean>,
) {
  const tree = computed<AccountRow[]>(() => {
    const filtered = showArchived.value
      ? accounts.value
      : accounts.value.filter((a) => !a.archivedAt);
    return buildAccountRows(filtered);
  });
  return { tree };
}
