import { defineConfig, type Plugin } from "vite";
import path from "path";
import { copyFile } from "fs/promises";

const copyFilePlugin = (): Plugin => {
  return {
    name: "copy-file-plugin",
    async writeBundle() {
      await copyFile("../../node_modules/bnf-parser/bnf.json", "./.vite/bnf.json");
    },
  };
};

// https://vitejs.dev/config
export default defineConfig({
  plugins: [copyFilePlugin()],
  resolve: {
    mainFields: ["browser", "module", "jsnext:main", "jsnext"],
    alias: {
      "@backend": path.resolve(__dirname, "../backend-core/src"),
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    rollupOptions: {
      external: ["sqlite3"],
    },
  },
});
