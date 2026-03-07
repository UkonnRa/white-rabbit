import type { JournalFilter } from "../../models";

export function useJournals(filter?: MaybeRef<JournalFilter>) {
  const client = useJournalClient();
  return useAsyncData("journals", () => client.list(toValue(filter)), {
    watch: filter
      ? [isRef(filter) ? filter : undefined].filter(Boolean)
      : undefined,
  });
}

export function useJournal(id: MaybeRef<string>) {
  const client = useJournalClient();
  return useAsyncData(`journal:${toValue(id)}`, () => client.get(toValue(id)));
}
