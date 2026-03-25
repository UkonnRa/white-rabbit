import type {
  Journal,
  CreateJournalRequest,
  UpdateJournalRequest,
  JournalFilter,
} from "~/models";

export interface JournalClient {
  create(request: CreateJournalRequest): Promise<Journal>;
  get(id: string): Promise<Journal>;
  list(filter?: JournalFilter): Promise<Journal[]>;
  update(id: string, request: UpdateJournalRequest): Promise<Journal>;
  delete(id: string): Promise<void>;
}

declare module "#app" {
  interface NuxtApp {
    $journalClient: JournalClient;
  }
}
