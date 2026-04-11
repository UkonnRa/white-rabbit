import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "jsdom",
    include: ["app/**/*.test.ts"],
    globals: true,
    server: {
      deps: {
        inline: ["@material/material-color-utilities"],
      },
    },
  },
  resolve: {
    alias: {
      "@": new URL("./app", import.meta.url).pathname,
    },
  },
});
