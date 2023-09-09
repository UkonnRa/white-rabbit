import { defineConfig } from "vite";
import path from "path";

// https://vitejs.dev/config
export default defineConfig({
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
