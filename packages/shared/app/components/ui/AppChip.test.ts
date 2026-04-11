import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppChip from "./AppChip.vue";

describe("AppChip component", () => {
  it("renders a span element", () => {
    const wrapper = mountWithTheme(AppChip, {
      slots: { default: "Tag" },
    });
    expect(wrapper.element.tagName).toBe("SPAN");
  });

  it("renders slot content", () => {
    const wrapper = mountWithTheme(AppChip, {
      slots: { default: "My Tag" },
    });
    expect(wrapper.text()).toContain("My Tag");
  });

  it("applies tonal variant by default", () => {
    const wrapper = mountWithTheme(AppChip, {
      slots: { default: "Tag" },
    });
    expect(wrapper.classes()).toContain("bg-surface-variant");
    expect(wrapper.classes()).toContain("text-on-surface-variant");
  });

  it("applies solid variant", () => {
    const wrapper = mountWithTheme(AppChip, {
      props: { variant: "solid" },
      slots: { default: "Tag" },
    });
    expect(wrapper.classes()).toContain("bg-primary");
  });

  it("applies outlined variant", () => {
    const wrapper = mountWithTheme(AppChip, {
      props: { variant: "outlined" },
      slots: { default: "Tag" },
    });
    expect(wrapper.classes()).toContain("border");
    expect(wrapper.classes()).toContain("border-outline");
  });

  it("applies sm size", () => {
    const wrapper = mountWithTheme(AppChip, {
      props: { size: "sm" },
      slots: { default: "Tag" },
    });
    expect(wrapper.classes()).toContain("h-6");
    expect(wrapper.classes()).toContain("text-xs");
  });

  it("does not render close button by default", () => {
    const wrapper = mountWithTheme(AppChip, {
      slots: { default: "Tag" },
    });
    expect(wrapper.find("button").exists()).toBe(false);
  });

  it("renders close button when closable", () => {
    const wrapper = mountWithTheme(AppChip, {
      props: { closable: true },
      slots: { default: "Tag" },
    });
    expect(wrapper.find("button").exists()).toBe(true);
  });

  it("emits close when close button clicked", async () => {
    const wrapper = mountWithTheme(AppChip, {
      props: { closable: true },
      slots: { default: "Tag" },
    });
    await wrapper.find("button").trigger("click");
    expect(wrapper.emitted("close")).toHaveLength(1);
  });
});
