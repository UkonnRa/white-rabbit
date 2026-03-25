import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import pluginVue from "eslint-plugin-vue";
import json from "@eslint/json";
import markdown from "@eslint/markdown";
import css from "@eslint/css";
import { defineConfig, globalIgnores } from "eslint/config";
import { fileURLToPath } from "node:url";
import { includeIgnoreFile } from "@eslint/compat";
import { Plugin } from "@eslint/core";
import eslintConfigPrettier from "eslint-config-prettier/flat";

const gitignorePath = fileURLToPath(new URL(".gitignore", import.meta.url));

export default defineConfig([
  includeIgnoreFile(gitignorePath, "Imported .gitignore patterns"),
  globalIgnores([".yarn"], "Ignore yarn cache directory"),
  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts}"],
    plugins: { js },
    extends: ["js/recommended"],
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  tseslint.configs.recommended,
  {
    files: ["**/*.vue"],
    extends: [...pluginVue.configs["flat/recommended"]],
    languageOptions: {
      parserOptions: { parser: tseslint.parser },
    },
    rules: {
      "vue/multi-word-component-names": 0,
    },
  },
  {
    files: ["**/*.json"],
    plugins: { json },
    language: "json/json",
    extends: ["json/recommended"],
  },
  {
    files: ["**/*.md"],
    plugins: { markdown: markdown as Plugin },
    language: "markdown/commonmark",
    extends: ["markdown/recommended"],
  },
  {
    files: ["**/*.css"],
    // @eslint/css cannot parse Tailwind v4 directives (@custom-variant, @theme inline, @apply)
    ignores: ["**/tailwind.css"],
    plugins: { css: css as Plugin },
    language: "css/css",
    extends: ["css/recommended"],
  },
  {
    files: ["**/components/JournalTable.vue"],
    rules: {
      "vue/valid-v-slot": ["error", { allowModifiers: true }],
    },
  },
  eslintConfigPrettier,
]);
