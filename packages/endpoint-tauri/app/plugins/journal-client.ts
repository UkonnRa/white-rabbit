import { TauriJournalClient } from "~/clients";

export default defineNuxtPlugin(() => ({
  provide: { journalClient: new TauriJournalClient() },
}));
