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
  it("accepts type field", () => {
    const req: CreateAccountRequest = {
      journal_id: "j1",
      parent_id: null,
      type: AccountType.Asset,
      name: "Cash",
    };
    expect(req.type).toBe(AccountType.Asset);
  });
});
