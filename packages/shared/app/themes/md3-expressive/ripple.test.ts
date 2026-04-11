import { describe, it, expect, beforeEach } from "vitest";
import { rippleDirective } from "./ripple";

interface RippleHTMLElement extends HTMLElement {
  _rippleCleanup?: () => void;
}

describe("ripple directive", () => {
  let el: RippleHTMLElement;

  beforeEach(() => {
    el = document.createElement("button") as RippleHTMLElement;
    el.style.width = "100px";
    el.style.height = "40px";
    document.body.appendChild(el);
  });

  it("sets position relative and overflow hidden on mounted", () => {
    rippleDirective.mounted!(el, {
      value: true,
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(el.style.position).toBe("relative");
    expect(el.style.overflow).toBe("hidden");
  });

  it("does not setup when disabled via boolean false", () => {
    rippleDirective.mounted!(el, {
      value: false,
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(el._rippleCleanup).toBeUndefined();
  });

  it("does not setup when disabled via object", () => {
    rippleDirective.mounted!(el, {
      value: { disabled: true },
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(el._rippleCleanup).toBeUndefined();
  });

  it("stores a cleanup function when enabled", () => {
    rippleDirective.mounted!(el, {
      value: true,
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(typeof el._rippleCleanup).toBe("function");
  });

  it("cleans up on unmount", () => {
    rippleDirective.mounted!(el, {
      value: true,
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(el._rippleCleanup).toBeDefined();

    rippleDirective.unmounted!(el, {
      value: true,
      modifiers: {},
      dir: rippleDirective,
      instance: null,
      oldValue: undefined,
    });
    expect(el._rippleCleanup).toBeUndefined();
  });
});
