import { describe, it, expect } from "vitest";
import { AccountType } from "./account";

describe("AccountType", () => {
  it("has the five root account types", () => {
    expect(AccountType.Asset).toBe("Asset");
    expect(AccountType.Liability).toBe("Liability");
    expect(AccountType.Equity).toBe("Equity");
    expect(AccountType.Income).toBe("Income");
    expect(AccountType.Expense).toBe("Expense");
  });
});
