import { describe, it, expect, beforeEach } from "vitest";
import type { ThemeDefinition } from "./types";
import { registerTheme, getTheme, listThemes, clearThemes } from "./registry";

const stubTheme: ThemeDefinition = {
  name: "stub",
  recipes: {
    button: {
      base: ["btn"],
      variants: { solid: ["bg-primary"] },
      sizes: { md: ["h-10"] },
      interactions: {},
    },
  },
};

describe("theme registry", () => {
  beforeEach(() => {
    clearThemes();
  });

  it("registers and retrieves a theme", () => {
    registerTheme(stubTheme);
    expect(getTheme("stub")).toBe(stubTheme);
  });

  it("throws on unknown theme with helpful message", () => {
    registerTheme(stubTheme);
    expect(() => getTheme("nonexistent")).toThrow(
      /Theme "nonexistent" not registered.*stub/,
    );
  });

  it("throws on empty registry", () => {
    expect(() => getTheme("any")).toThrow(/\(none\)/);
  });

  it("lists registered theme names", () => {
    registerTheme(stubTheme);
    registerTheme({ ...stubTheme, name: "another" });
    expect(listThemes()).toEqual(["stub", "another"]);
  });

  it("clearThemes removes all entries", () => {
    registerTheme(stubTheme);
    clearThemes();
    expect(listThemes()).toEqual([]);
  });
});
