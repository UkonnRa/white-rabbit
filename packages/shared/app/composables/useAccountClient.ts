import type { AccountClient } from "../clients";

export function useAccountClient(): AccountClient {
  return useNuxtApp().$accountClient;
}
