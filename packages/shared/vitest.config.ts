import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    include: ["app/**/*.test.ts"],
    globals: true,
  },
  resolve: {
    alias: {
      "@": new URL("./app", import.meta.url).pathname,
    },
  },
});
