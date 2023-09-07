import { defineConfig } from "vite";
import path from "path";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@frontend": path.resolve(__dirname, "../frontend-core/src"),
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
