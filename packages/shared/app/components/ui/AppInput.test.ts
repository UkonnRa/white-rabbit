import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppInput from "./AppInput.vue";

describe("AppInput component", () => {
  it("renders an input element", () => {
    const wrapper = mountWithTheme(AppInput);
    expect(wrapper.element.tagName).toBe("INPUT");
  });

  it("applies default variant + md size classes", () => {
    const wrapper = mountWithTheme(AppInput, {
      props: { variant: "default", size: "md" },
    });
    expect(wrapper.classes()).toContain("rounded-md");
    expect(wrapper.classes()).toContain("border");
    expect(wrapper.classes()).toContain("bg-surface");
    expect(wrapper.classes()).toContain("h-10");
  });

  it("applies error variant classes", () => {
    const wrapper = mountWithTheme(AppInput, {
      props: { variant: "error" },
    });
    expect(wrapper.classes()).toContain("border-error");
  });

  it("applies sm size classes", () => {
    const wrapper = mountWithTheme(AppInput, {
      props: { size: "sm" },
    });
    expect(wrapper.classes()).toContain("h-8");
    expect(wrapper.classes()).toContain("px-2.5");
  });

  it("sets disabled attribute and classes", () => {
    const wrapper = mountWithTheme(AppInput, {
      props: { disabled: true },
    });
    expect(wrapper.attributes("disabled")).toBeDefined();
    expect(wrapper.classes()).toContain("opacity-50");
    expect(wrapper.classes()).toContain("cursor-not-allowed");
  });

  it("emits update:modelValue on input", async () => {
    const onUpdate = vi.fn();
    const wrapper = mountWithTheme(AppInput, {
      props: { modelValue: "", "onUpdate:modelValue": onUpdate },
    });
    await wrapper.setValue("hello");
    expect(onUpdate).toHaveBeenCalled();
  });

  it("passes through placeholder attribute", () => {
    const wrapper = mountWithTheme(AppInput, {
      attrs: { placeholder: "Type here…" },
    });
    expect(wrapper.attributes("placeholder")).toBe("Type here…");
  });
});
