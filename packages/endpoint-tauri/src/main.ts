import { createApp } from "vue";
import { TauriJournalClient } from "./clients";
import App from "./App.vue";

const app = createApp(App);

app.provide("journalClient", new TauriJournalClient());

app.mount("#app");
