import { describe, it, expect } from "vitest";
import { AccountType } from "./account";
import type { CreateAccountRequest } from "./account";

describe("AccountType", () => {
  it("has the five root account types", () => {
    expect(AccountType.Asset).toBe("Asset");
    expect(AccountType.Liability).toBe("Liability");
    expect(AccountType.Equity).toBe("Equity");
    expect(AccountType.Income).toBe("Income");
    expect(AccountType.Expense).toBe("Expense");
  });
});

describe("CreateAccountRequest", () => {
  it("accepts required fields", () => {
    const req: CreateAccountRequest = {
      journalId: "j1",
      parentId: "p1",
      name: "Cash",
    };
    expect(req.name).toBe("Cash");
  });
});
