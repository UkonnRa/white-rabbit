import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppButton from "./AppButton.vue";

describe("AppButton component", () => {
  it("renders a button element by default", () => {
    const wrapper = mountWithTheme(AppButton);
    expect(wrapper.element.tagName).toBe("BUTTON");
  });

  it("renders slot content", () => {
    const wrapper = mountWithTheme(AppButton, {
      slots: { default: "Click me" },
    });
    expect(wrapper.text()).toBe("Click me");
  });

  it("applies solid variant classes from recipe", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { variant: "solid", size: "md" },
    });
    expect(wrapper.classes()).toContain("bg-primary");
    expect(wrapper.classes()).toContain("text-on-primary");
    expect(wrapper.classes()).toContain("h-10");
  });

  it("applies outlined variant classes", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { variant: "outlined" },
    });
    expect(wrapper.classes()).toContain("border");
    expect(wrapper.classes()).toContain("border-outline");
    expect(wrapper.classes()).toContain("text-primary");
  });

  it("applies ghost variant classes", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { variant: "ghost" },
    });
    expect(wrapper.classes()).toContain("bg-transparent");
    expect(wrapper.classes()).toContain("text-on-surface");
  });

  it("applies text variant classes", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { variant: "text" },
    });
    expect(wrapper.classes()).toContain("text-primary");
  });

  it("applies sm size classes", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { size: "sm" },
    });
    expect(wrapper.classes()).toContain("h-8");
    expect(wrapper.classes()).toContain("px-3");
  });

  it("applies lg size classes", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { size: "lg" },
    });
    expect(wrapper.classes()).toContain("h-12");
    expect(wrapper.classes()).toContain("px-6");
  });

  it("applies disabled classes and sets disabled attribute", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { disabled: true },
    });
    expect(wrapper.attributes("disabled")).toBeDefined();
    expect(wrapper.classes()).toContain("opacity-50");
    expect(wrapper.classes()).toContain("pointer-events-none");
    expect(wrapper.classes()).not.toContain("hover:brightness-110");
  });

  it("includes interaction classes when not disabled", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { variant: "solid", size: "md" },
    });
    expect(wrapper.classes()).toContain("hover:brightness-110");
    expect(wrapper.classes()).toContain("focus-visible:outline-2");
    expect(wrapper.classes()).toContain("active:scale-[0.98]");
  });

  it("emits click event", async () => {
    const wrapper = mountWithTheme(AppButton, {
      slots: { default: "Go" },
    });
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toHaveLength(1);
  });

  it("renders as a different element via as prop", () => {
    const wrapper = mountWithTheme(AppButton, {
      props: { as: "a" },
    });
    expect(wrapper.element.tagName).toBe("A");
  });
});
