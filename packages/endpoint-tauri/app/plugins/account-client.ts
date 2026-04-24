import { TauriAccountClient } from "~/clients";

export default defineNuxtPlugin(() => ({
  provide: { accountClient: new TauriAccountClient() },
}));
