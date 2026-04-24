import { describe, it, expect } from "vitest";
import { buildAccountRows } from "./account";
import type { Account } from "../models";
import { AccountType } from "../models";

function makeAcc(overrides: Partial<Account> & { id: string }): Account {
  return {
    journal_id: "j1",
    parent_id: null,
    type: AccountType.Asset,
    name: "Test",
    description: "",
    tags: [],
    created_at: null,
    last_modified_at: null,
    archived_at: null,
    ...overrides,
  };
}

describe("buildAccountRows", () => {
  it("returns 5 root rows from the 5 root accounts", () => {
    const accounts: Account[] = [
      makeAcc({ id: "a1", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "l1", type: AccountType.Liability, parent_id: null, name: "Liability" }),
      makeAcc({ id: "e1", type: AccountType.Equity, parent_id: null, name: "Equity" }),
      makeAcc({ id: "i1", type: AccountType.Income, parent_id: null, name: "Income" }),
      makeAcc({ id: "x1", type: AccountType.Expense, parent_id: null, name: "Expense" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows).toHaveLength(5);
    expect(rows.map(r => r.type)).toEqual([
      AccountType.Asset,
      AccountType.Liability,
      AccountType.Equity,
      AccountType.Income,
      AccountType.Expense,
    ]);
  });

  it("nests children under parents via subRows", () => {
    const accounts: Account[] = [
      makeAcc({ id: "root", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "child1", type: AccountType.Asset, parent_id: "root", name: "Bank" }),
      makeAcc({ id: "child2", type: AccountType.Asset, parent_id: "root", name: "Cash" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows).toHaveLength(1);
    expect(rows[0].subRows).toHaveLength(2);
    expect(rows[0].subRows[0].name).toBe("Bank");
    expect(rows[0].subRows[1].name).toBe("Cash");
  });

  it("computes depth correctly", () => {
    const accounts: Account[] = [
      makeAcc({ id: "root", type: AccountType.Asset, parent_id: null, name: "Asset" }),
      makeAcc({ id: "c1", type: AccountType.Asset, parent_id: "root", name: "Bank" }),
      makeAcc({ id: "c2", type: AccountType.Asset, parent_id: "c1", name: "Checking" }),
    ];
    const rows = buildAccountRows(accounts);
    expect(rows[0].depth).toBe(0);
    expect(rows[0].subRows[0].depth).toBe(1);
    expect(rows[0].subRows[0].subRows[0].depth).toBe(2);
  });

  it("handles empty account list", () => {
    const rows = buildAccountRows([]);
    expect(rows).toHaveLength(0);
  });
});
