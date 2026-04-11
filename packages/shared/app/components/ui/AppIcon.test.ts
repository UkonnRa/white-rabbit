import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../../test-utils-mount";
import AppIcon from "./AppIcon.vue";

describe("AppIcon component", () => {
  it("renders with required icon prop", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check" },
    });
    expect(wrapper.exists()).toBe(true);
  });

  it("has aria-hidden attribute", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check" },
    });
    expect(wrapper.attributes("aria-hidden")).toBe("true");
  });

  it("applies recipe classes", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check" },
    });
    expect(wrapper.classes()).toContain("inline-flex");
    expect(wrapper.classes()).toContain("shrink-0");
  });

  it("passes icon prop", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:star" },
    });
    expect(wrapper.props("icon")).toBe("lucide:star");
  });

  it("sets width and height based on size", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check", size: "lg" },
    });
    expect(wrapper.attributes("width")).toBe("24");
    expect(wrapper.attributes("height")).toBe("24");
  });

  it("defaults to md size (20px)", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check" },
    });
    expect(wrapper.attributes("width")).toBe("20");
    expect(wrapper.attributes("height")).toBe("20");
  });

  it("applies sm size (16px)", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check", size: "sm" },
    });
    expect(wrapper.attributes("width")).toBe("16");
    expect(wrapper.attributes("height")).toBe("16");
  });

  it("applies xl size (32px)", () => {
    const wrapper = mountWithTheme(AppIcon, {
      props: { icon: "lucide:check", size: "xl" },
    });
    expect(wrapper.attributes("width")).toBe("32");
    expect(wrapper.attributes("height")).toBe("32");
  });
});
