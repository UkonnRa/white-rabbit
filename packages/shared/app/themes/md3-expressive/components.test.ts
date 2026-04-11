import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppButton from "../../components/ui/AppButton.vue";
import AppCard from "../../components/ui/AppCard.vue";
import AppChip from "../../components/ui/AppChip.vue";
import AppInput from "../../components/ui/AppInput.vue";

describe("components under md3-expressive theme", () => {
  it("AppButton uses rounded-full (MD3 shape)", () => {
    const wrapper = mountWithTheme(AppButton, {
      theme: "md3-expressive",
      slots: { default: "Click me" },
    });
    expect(wrapper.classes()).toContain("rounded-full");
    expect(wrapper.classes()).toContain("md3-state-layer");
  });

  it("AppButton does not use tailwind-default's rounded-md", () => {
    const wrapper = mountWithTheme(AppButton, {
      theme: "md3-expressive",
      slots: { default: "Click me" },
    });
    expect(wrapper.classes()).not.toContain("rounded-md");
  });

  it("AppCard uses rounded-xl (MD3 shape)", () => {
    const wrapper = mountWithTheme(AppCard, {
      theme: "md3-expressive",
      slots: { default: "Content" },
    });
    expect(wrapper.classes()).toContain("rounded-xl");
  });

  it("AppChip uses md3-state-layer", () => {
    const wrapper = mountWithTheme(AppChip, {
      theme: "md3-expressive",
      slots: { default: "Tag" },
    });
    expect(wrapper.classes()).toContain("md3-state-layer");
    expect(wrapper.classes()).toContain("rounded-full");
  });

  it("AppInput uses rounded-xl and surface-container bg", () => {
    const wrapper = mountWithTheme(AppInput, {
      theme: "md3-expressive",
    });
    expect(wrapper.classes()).toContain("rounded-xl");
    expect(wrapper.classes()).toContain("bg-surface-container");
  });

  it("components still render correctly under tailwind-default", () => {
    const wrapper = mountWithTheme(AppButton, {
      theme: "tailwind-default",
      slots: { default: "Click me" },
    });
    expect(wrapper.classes()).toContain("rounded-md");
    expect(wrapper.classes()).not.toContain("md3-state-layer");
  });
});
