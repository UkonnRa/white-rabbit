import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppTextarea from "./AppTextarea.vue";

describe("AppTextarea component", () => {
  it("renders a textarea element", () => {
    const wrapper = mountWithTheme(AppTextarea);
    expect(wrapper.element.tagName).toBe("TEXTAREA");
  });

  it("applies recipe classes", () => {
    const wrapper = mountWithTheme(AppTextarea, {
      props: { size: "md" },
    });
    expect(wrapper.classes()).toContain("rounded-md");
    expect(wrapper.classes()).toContain("bg-surface");
    expect(wrapper.classes()).toContain("resize-y");
  });

  it("sets rows attribute", () => {
    const wrapper = mountWithTheme(AppTextarea, {
      props: { rows: 5 },
    });
    expect(wrapper.attributes("rows")).toBe("5");
  });

  it("defaults to 3 rows", () => {
    const wrapper = mountWithTheme(AppTextarea);
    expect(wrapper.attributes("rows")).toBe("3");
  });

  it("sets disabled attribute and classes", () => {
    const wrapper = mountWithTheme(AppTextarea, {
      props: { disabled: true },
    });
    expect(wrapper.attributes("disabled")).toBeDefined();
    expect(wrapper.classes()).toContain("opacity-50");
  });
});
