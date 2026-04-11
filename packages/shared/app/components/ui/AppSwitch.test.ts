import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppSwitch from "./AppSwitch.vue";

describe("AppSwitch component", () => {
  it("renders a button with switch role", () => {
    const wrapper = mountWithTheme(AppSwitch);
    const button = wrapper.find("button");
    expect(button.exists()).toBe(true);
    expect(button.attributes("role")).toBe("switch");
  });

  it("applies root recipe classes", () => {
    const wrapper = mountWithTheme(AppSwitch, {
      props: { size: "md" },
    });
    const button = wrapper.find("button");
    expect(button.classes()).toContain("rounded-full");
    expect(button.classes()).toContain("h-6");
    expect(button.classes()).toContain("w-11");
  });

  it("renders thumb element", () => {
    const wrapper = mountWithTheme(AppSwitch);
    const thumb = wrapper.find("span");
    expect(thumb.exists()).toBe(true);
    expect(thumb.classes()).toContain("rounded-full");
  });

  it("applies sm size", () => {
    const wrapper = mountWithTheme(AppSwitch, {
      props: { size: "sm" },
    });
    const button = wrapper.find("button");
    expect(button.classes()).toContain("h-5");
    expect(button.classes()).toContain("w-9");
  });

  it("defaults to unchecked state", () => {
    const wrapper = mountWithTheme(AppSwitch);
    const button = wrapper.find("button");
    expect(button.attributes("aria-checked")).toBe("false");
    expect(button.attributes("data-state")).toBe("unchecked");
  });

  it("has correct role and keyboard accessibility", () => {
    const wrapper = mountWithTheme(AppSwitch);
    const button = wrapper.find("button");
    expect(button.attributes("role")).toBe("switch");
    expect(button.attributes("type")).toBe("button");
  });
});
