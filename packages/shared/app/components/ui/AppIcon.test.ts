import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppIcon from "./AppIcon.vue";

describe("AppIcon component", () => {
  it("renders a span element", () => {
    const wrapper = mountWithTheme(AppIcon);
    expect(wrapper.element.tagName).toBe("SPAN");
  });

  it("has aria-hidden attribute", () => {
    const wrapper = mountWithTheme(AppIcon);
    expect(wrapper.attributes("aria-hidden")).toBe("true");
  });

  it("renders slot content", () => {
    const wrapper = mountWithTheme(AppIcon, {
      slots: { default: "★" },
    });
    expect(wrapper.text()).toBe("★");
  });

  it("applies md size by default", () => {
    const wrapper = mountWithTheme(AppIcon);
    expect(wrapper.classes()).toContain("size-5");
  });

  it("applies sm size", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { size: "sm" },
    });
    expect(wrapper.classes()).toContain("size-4");
  });

  it("applies lg size", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { size: "lg" },
    });
    expect(wrapper.classes()).toContain("size-6");
  });

  it("applies xl size", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { size: "xl" },
    });
    expect(wrapper.classes()).toContain("size-8");
  });

  it("includes base classes", () => {
    const wrapper = mountWithTheme(AppIcon);
    expect(wrapper.classes()).toContain("inline-flex");
    expect(wrapper.classes()).toContain("shrink-0");
  });
});
