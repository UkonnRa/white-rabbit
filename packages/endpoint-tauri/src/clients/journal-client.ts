import { invoke } from "@tauri-apps/api/core";
import type {
  JournalClient,
  Journal,
  CreateJournalRequest,
  UpdateJournalRequest,
  JournalFilter,
} from "@white-rabbit/shared";

export class TauriJournalClient implements JournalClient {
  async create(request: CreateJournalRequest): Promise<Journal> {
    return invoke("create_journal", { request });
  }

  async get(id: string): Promise<Journal> {
    return invoke("get_journal", { id });
  }

  async list(filter?: JournalFilter): Promise<Journal[]> {
    return invoke("list_journals", { filter: filter ?? {} });
  }

  async update(id: string, request: UpdateJournalRequest): Promise<Journal> {
    return invoke("update_journal", { id, request });
  }

  async delete(id: string): Promise<void> {
    return invoke("delete_journal", { id });
  }
}
