import { describe, it, expect, beforeEach } from "vitest";
import { nextTick } from "vue";
import { useMode, _resetMode } from "./useMode";
import { withSetup } from "../test-utils";

beforeEach(() => {
  delete document.documentElement.dataset.mode;
  _resetMode("light");
});

describe("useMode", () => {
  it("defaults to light", () => {
    const { result, unmount } = withSetup(() => useMode());
    expect(result.mode.value).toBe("light");
    unmount();
  });

  it("returns the same singleton ref across calls", () => {
    const { result: a, unmount: u1 } = withSetup(() => useMode());
    const { result: b, unmount: u2 } = withSetup(() => useMode());
    expect(a.mode).toBe(b.mode);
    u1();
    u2();
  });

  it("sets data-mode on <html>", async () => {
    const { result, unmount } = withSetup(() => useMode());
    result.mode.value = "dark";
    await nextTick();
    expect(document.documentElement.dataset.mode).toBe("dark");
    unmount();
  });

  it("toggle flips light to dark", async () => {
    const { result, unmount } = withSetup(() => useMode());
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("dark");
    expect(document.documentElement.dataset.mode).toBe("dark");
    unmount();
  });

  it("toggle flips dark to light", async () => {
    _resetMode("dark");
    const { result, unmount } = withSetup(() => useMode());
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("light");
    expect(document.documentElement.dataset.mode).toBe("light");
    unmount();
  });

  it("double toggle returns to original", async () => {
    const { result, unmount } = withSetup(() => useMode());
    result.toggle();
    result.toggle();
    await nextTick();

    expect(result.mode.value).toBe("light");
    unmount();
  });
});
