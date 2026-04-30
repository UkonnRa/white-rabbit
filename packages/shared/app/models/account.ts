export enum AccountType {
  Asset = "Asset",
  Liability = "Liability",
  Equity = "Equity",
  Income = "Income",
  Expense = "Expense",
}

export interface Account {
  id: string;
  journalId: string;
  parentId: string | null;
  type: AccountType;
  name: string;
  description: string;
  tags: string[];
  createdAt: string | null;
  lastModifiedAt: string | null;
  archivedAt: string | null;
}

export interface CreateAccountRequest {
  journalId: string;
  parentId: string;
  name: string;
  description?: string;
  tags?: string[];
}

export interface UpdateAccountRequest {
  name?: string;
  description?: string;
  tags?: string[];
}

export interface AccountFilter {
  id?: string;
  journalId?: string;
  parentId?: string;
  name?: string;
  type?: AccountType | string;
  tag?: string;
  fullText?: string;
}

export interface AccountFormData {
  name: string;
  description: string;
  tags: string[];
}

export interface AccountRow extends Account {
  subRows: AccountRow[];
  depth: number;
}
