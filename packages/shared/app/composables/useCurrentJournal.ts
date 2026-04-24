export function useCurrentJournal() {
  const route = useRoute();
  const journalId = computed(() => route.params.id as string);
  const { data: journal, refresh, status } = useJournal(journalId);
  return { journal, journalId, refresh, status };
}
