import { invoke } from "@tauri-apps/api/core";
import type {
  AccountClient,
  Account,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountFilter,
} from "@white-rabbit/shared";

export class TauriAccountClient implements AccountClient {
  async create(request: CreateAccountRequest): Promise<Account> {
    return invoke("create_account", { request });
  }

  async get(id: string): Promise<Account> {
    return invoke("get_account", { id });
  }

  async list(filter?: AccountFilter): Promise<Account[]> {
    return invoke("list_accounts", { filter: filter ?? {} });
  }

  async update(id: string, request: UpdateAccountRequest): Promise<Account> {
    return invoke("update_account", { id, request });
  }

  async delete(id: string): Promise<void> {
    return invoke("delete_account", { id });
  }

  async archive(id: string): Promise<Account> {
    return invoke("archive_account", { id });
  }
}
