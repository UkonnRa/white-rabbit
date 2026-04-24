import type {
  Account,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountFilter,
} from "../models";

export interface AccountClient {
  create(request: CreateAccountRequest): Promise<Account>;
  get(id: string): Promise<Account>;
  list(filter?: AccountFilter): Promise<Account[]>;
  update(id: string, request: UpdateAccountRequest): Promise<Account>;
  delete(id: string): Promise<void>;
  archive(id: string): Promise<Account>;
}

declare module "#app" {
  interface NuxtApp {
    $accountClient: AccountClient;
  }
}
