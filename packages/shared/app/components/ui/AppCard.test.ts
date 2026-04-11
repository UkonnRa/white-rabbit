import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppCard from "./AppCard.vue";

describe("AppCard component", () => {
  it("renders a div by default", () => {
    const wrapper = mountWithTheme(AppCard);
    expect(wrapper.element.tagName).toBe("DIV");
  });

  it("renders slot content", () => {
    const wrapper = mountWithTheme(AppCard, {
      slots: { default: "<p>Card content</p>" },
    });
    expect(wrapper.text()).toContain("Card content");
  });

  it("applies elevated variant classes", () => {
    const wrapper = mountWithTheme(AppCard, {
      props: { variant: "elevated" },
    });
    expect(wrapper.classes()).toContain("shadow-md");
    expect(wrapper.classes()).toContain("rounded-lg");
    expect(wrapper.classes()).toContain("bg-surface");
  });

  it("applies outlined variant classes", () => {
    const wrapper = mountWithTheme(AppCard, {
      props: { variant: "outlined" },
    });
    expect(wrapper.classes()).toContain("border");
    expect(wrapper.classes()).toContain("border-outline-variant");
  });

  it("applies filled variant classes", () => {
    const wrapper = mountWithTheme(AppCard, {
      props: { variant: "filled" },
    });
    expect(wrapper.classes()).toContain("bg-surface-variant");
  });

  it("applies size classes", () => {
    const wrapper = mountWithTheme(AppCard, {
      props: { size: "lg" },
    });
    expect(wrapper.classes()).toContain("p-6");
  });

  it("renders as a different element via as prop", () => {
    const wrapper = mountWithTheme(AppCard, {
      props: { as: "section" },
    });
    expect(wrapper.element.tagName).toBe("SECTION");
  });
});
