import type { JournalFilter } from "../models";

export function useJournals(filter?: MaybeRef<JournalFilter>) {
  const client = useJournalClient();
  return useAsyncData("journals", () => client.list(toValue(filter)), {
    watch: isRef(filter) ? [filter] : undefined,
  });
}

export function useJournal(id: MaybeRef<string>) {
  const client = useJournalClient();
  return useAsyncData(`journal:${toValue(id)}`, () => client.get(toValue(id)), {
    watch: isRef(id) ? [id] : undefined,
  });
}
