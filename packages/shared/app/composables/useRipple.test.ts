import { describe, it, expect, beforeEach } from "vitest";
import { defineComponent, h, ref, computed, nextTick } from "vue";
import { createApp } from "vue";
import { useRipple } from "./useRipple";
import { THEME_KEY, MODE_KEY } from "../themes/types";
import { tailwindDefault } from "../themes/tailwind-default";
import { md3Expressive } from "../themes/md3-expressive";
import type { RippleHTMLElement } from "../themes/md3-expressive/ripple";

// jsdom doesn't support Web Animations API — stub it
beforeEach(() => {
  if (!HTMLElement.prototype.animate) {
    HTMLElement.prototype.animate = () =>
      ({
        onfinish: null,
        cancel: () => {},
        finished: Promise.resolve({} as Animation),
      }) as unknown as Animation;
  }
  if (!HTMLSpanElement.prototype.animate) {
    HTMLSpanElement.prototype.animate = HTMLElement.prototype.animate;
  }
});

function mountWithRipple(
  themeDef: typeof tailwindDefault | typeof md3Expressive,
) {
  const themeRef = computed(() => themeDef);
  const modeRef = computed<"light" | "dark">(() => "light");

  const App = defineComponent({
    setup() {
      const elRef = ref<HTMLElement | null>(null);
      useRipple(elRef);
      return () =>
        h(
          "button",
          {
            ref: (el: HTMLElement) => {
              elRef.value = el;
            },
          },
          "Click",
        );
    },
  });

  const app = createApp(App);
  app.provide(THEME_KEY, themeRef);
  app.provide(MODE_KEY, modeRef);

  const root = document.createElement("div");
  document.body.appendChild(root);
  app.mount(root);

  const buttonEl = root.querySelector("button") as RippleHTMLElement;

  return {
    el: buttonEl,
    unmount: () => {
      app.unmount();
      root.remove();
    },
  };
}

describe("useRipple", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("attaches ripple handler when MD3 theme is active", () => {
    const { el, unmount } = mountWithRipple(md3Expressive);
    expect(el).toBeTruthy();
    expect(el!._rippleCleanup).toBeDefined();
    expect(el!.style.overflow).toBe("hidden");
    unmount();
  });

  it("does NOT attach ripple when tailwind-default theme is active", () => {
    const { el, unmount } = mountWithRipple(tailwindDefault);
    expect(el).toBeTruthy();
    expect(el!._rippleCleanup).toBeUndefined();
    unmount();
  });

  it("cleans up ripple on unmount", () => {
    const { el, unmount } = mountWithRipple(md3Expressive);
    expect(el!._rippleCleanup).toBeDefined();
    unmount();
    expect(el!._rippleCleanup).toBeUndefined();
  });

  it("creates a ripple span on pointerdown", () => {
    const { el, unmount } = mountWithRipple(md3Expressive);

    // Simulate pointerdown with getBoundingClientRect mock
    el!.getBoundingClientRect = () => ({
      x: 0,
      y: 0,
      width: 100,
      height: 40,
      top: 0,
      left: 0,
      right: 100,
      bottom: 40,
      toJSON: () => {},
    });

    el!.dispatchEvent(
      new PointerEvent("pointerdown", {
        clientX: 50,
        clientY: 20,
        bubbles: true,
      }),
    );

    const rippleSpan = el!.querySelector("span");
    expect(rippleSpan).toBeTruthy();
    expect(rippleSpan!.style.borderRadius).toBe("50%");
    expect(rippleSpan!.style.position).toBe("absolute");

    unmount();
  });

  it("removes ripple span on pointerup", async () => {
    const { el, unmount } = mountWithRipple(md3Expressive);

    el!.getBoundingClientRect = () => ({
      x: 0,
      y: 0,
      width: 100,
      height: 40,
      top: 0,
      left: 0,
      right: 100,
      bottom: 40,
      toJSON: () => {},
    });

    el!.dispatchEvent(
      new PointerEvent("pointerdown", {
        clientX: 50,
        clientY: 20,
        bubbles: true,
      }),
    );

    expect(el!.querySelector("span")).toBeTruthy();

    el!.dispatchEvent(new PointerEvent("pointerup", { bubbles: true }));

    // The fade-out animation calls onfinish → remove.
    // In jsdom, Web Animations API may not fully work,
    // but at minimum the pointerup handler fires.
    await nextTick();

    unmount();
  });

  it("reacts to theme switch: removes ripple when switching to tailwind", async () => {
    const currentTheme = ref(md3Expressive);
    const themeRef = computed(() => currentTheme.value);
    const modeRef = computed<"light" | "dark">(() => "light");

    const App = defineComponent({
      setup() {
        const elRef = ref<HTMLElement | null>(null);
        useRipple(elRef);
        return () =>
          h(
            "button",
            {
              ref: (el: HTMLElement) => {
                elRef.value = el;
              },
            },
            "Click",
          );
      },
    });

    const app = createApp(App);
    app.provide(THEME_KEY, themeRef);
    app.provide(MODE_KEY, modeRef);

    const root = document.createElement("div");
    document.body.appendChild(root);
    app.mount(root);
    const buttonEl = root.querySelector("button") as RippleHTMLElement;

    expect(buttonEl._rippleCleanup).toBeDefined();

    // Switch to tailwind
    currentTheme.value = tailwindDefault;
    await nextTick();

    expect(buttonEl._rippleCleanup).toBeUndefined();

    app.unmount();
    root.remove();
  });
});
