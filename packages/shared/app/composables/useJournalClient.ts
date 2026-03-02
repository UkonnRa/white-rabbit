import type { JournalClient } from "../../clients";

export function useJournalClient(): JournalClient {
  return useNuxtApp().$journalClient;
}
