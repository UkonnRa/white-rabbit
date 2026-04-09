import { ref, watchEffect } from "vue";

const STORAGE_KEY = "wr-color-mode";

function readStorage(): "light" | "dark" | null {
  try {
    return globalThis.localStorage?.getItem?.(STORAGE_KEY) as
      | "light"
      | "dark"
      | null;
  } catch {
    return null;
  }
}

function writeStorage(value: string): void {
  try {
    globalThis.localStorage?.setItem?.(STORAGE_KEY, value);
  } catch {
    // Storage unavailable (SSR, security restrictions, Node.js stub).
  }
}

/**
 * Light/dark mode toggle with localStorage persistence.
 *
 * Syncs `data-mode` on `<html>` whenever the value changes.
 */
export function useMode(initial: "light" | "dark" = "light") {
  const mode = ref<"light" | "dark">(readStorage() ?? initial);

  watchEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset.mode = mode.value;
    }
    writeStorage(mode.value);
  });

  function toggle() {
    mode.value = mode.value === "light" ? "dark" : "light";
  }

  return { mode, toggle };
}
