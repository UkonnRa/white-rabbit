import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../test-utils-mount";
import AccountTable from "./AccountTable.vue";
import { AccountType } from "../models";
import type { AccountRow } from "../models";

function makeRow(overrides: Partial<AccountRow> & { id: string }): AccountRow {
  return {
    journalId: "j1",
    parentId: null,
    type: AccountType.Asset,
    name: "Asset",
    description: "",
    tags: [],
    createdAt: null,
    lastModifiedAt: null,
    archivedAt: null,
    subRows: [],
    depth: 0,
    ...overrides,
  };
}

describe("AccountTable", () => {
  it("renders account names", () => {
    const data: AccountRow[] = [
      makeRow({ id: "a1", type: AccountType.Asset, name: "Asset" }),
    ];
    const wrapper = mountWithTheme(AccountTable, { props: { data } });
    expect(wrapper.text()).toContain("Asset");
  });

  it("shows archived badge for archived accounts", () => {
    const data: AccountRow[] = [
      makeRow({
        id: "c1",
        name: "Bank",
        parentId: "p1",
        archivedAt: "2024-01-01",
      }),
    ];
    const wrapper = mountWithTheme(AccountTable, { props: { data } });
    expect(wrapper.text()).toContain("Archived");
  });

  it("renders child rows via subRows", async () => {
    const data: AccountRow[] = [
      makeRow({
        id: "root",
        name: "Asset",
        subRows: [
          makeRow({ id: "child1", name: "Bank", parentId: "root", depth: 1 }),
        ],
      }),
    ];
    const wrapper = mountWithTheme(AccountTable, { props: { data } });

    expect(wrapper.text()).toContain("Asset");
  });
});
