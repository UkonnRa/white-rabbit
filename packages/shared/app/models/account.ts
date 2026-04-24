export enum AccountType {
  Asset = "Asset",
  Liability = "Liability",
  Equity = "Equity",
  Income = "Income",
  Expense = "Expense",
}

export interface Account {
  id: string;
  journal_id: string;
  parent_id: string | null;
  type: AccountType;
  name: string;
  description: string;
  tags: string[];
  created_at: string | null;
  last_modified_at: string | null;
  archived_at: string | null;
}

export interface CreateAccountRequest {
  journal_id: string;
  parent_id: string;
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
  journal_id?: string;
  parent_id?: string;
  name?: string;
  type?: string;
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
