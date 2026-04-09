import { describe, it, expect, beforeEach } from "vitest";
import { nextTick } from "vue";
import { useMode } from "./useMode";
import { withSetup } from "../test-utils";

beforeEach(() => {
  delete document.documentElement.dataset.mode;
});

describe("useMode", () => {
  it("defaults to light", () => {
    const { result, unmount } = withSetup(() => useMode());
    expect(result.mode.value).toBe("light");
    unmount();
  });

  it("respects initial parameter", () => {
    const { result, unmount } = withSetup(() => useMode("dark"));
    expect(result.mode.value).toBe("dark");
    unmount();
  });

  it("sets data-mode on <html>", () => {
    const { unmount } = withSetup(() => useMode("dark"));
    expect(document.documentElement.dataset.mode).toBe("dark");
    unmount();
  });

  it("toggle flips light to dark", async () => {
    const { result, unmount } = withSetup(() => useMode("light"));
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("dark");
    expect(document.documentElement.dataset.mode).toBe("dark");
    unmount();
  });

  it("toggle flips dark to light", async () => {
    const { result, unmount } = withSetup(() => useMode("dark"));
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("light");
    expect(document.documentElement.dataset.mode).toBe("light");
    unmount();
  });

  it("double toggle returns to original", async () => {
    const { result, unmount } = withSetup(() => useMode("light"));
    result.toggle();
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("light");
    unmount();
  });
});
