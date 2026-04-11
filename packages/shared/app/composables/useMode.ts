import { ref, watch } from "vue";

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

// Module-level singleton so every caller shares the same reactive ref.
// The watcher is registered at module scope (not inside a component setup),
// so it is never disposed when components unmount.
const mode = ref<"light" | "dark">(readStorage() ?? "light");

watch(
  mode,
  (val) => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset.mode = val;
    }
    writeStorage(val);
  },
  { immediate: true },
);

/**
 * Light/dark mode toggle with localStorage persistence.
 *
 * Syncs `data-mode` on `<html>` whenever the value changes.
 * All callers share a single reactive ref (singleton).
 */
export function useMode() {
  function toggle() {
    mode.value = mode.value === "light" ? "dark" : "light";
  }

  return { mode, toggle };
}

/** Reset singleton state — for tests only. */
export function _resetMode(value: "light" | "dark" = "light") {
  mode.value = value;
}
