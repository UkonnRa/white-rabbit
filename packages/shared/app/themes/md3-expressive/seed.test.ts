import { describe, it, expect, beforeEach } from "vitest";
import {
  generateMD3Tokens,
  applyMD3TokensToElement,
  clearMD3TokensFromElement,
} from "./seed";

describe("generateMD3Tokens", () => {
  it("returns a record of color tokens for a seed hex", () => {
    const tokens = generateMD3Tokens("#6750A4", false);
    expect(tokens).toBeDefined();
    expect(typeof tokens).toBe("object");
    expect(Object.keys(tokens).length).toBeGreaterThan(0);
  });

  it("generates different tokens for light vs dark", () => {
    const light = generateMD3Tokens("#6750A4", false);
    const dark = generateMD3Tokens("#6750A4", true);
    // Primary color should differ between light and dark
    expect(light.primary).not.toBe(dark.primary);
  });

  it("generates different tokens for different seeds", () => {
    const purple = generateMD3Tokens("#6750A4", false);
    const blue = generateMD3Tokens("#1565C0", false);
    expect(purple.primary).not.toBe(blue.primary);
  });

  it("includes key MD3 color roles", () => {
    const tokens = generateMD3Tokens("#6750A4", false);
    expect(tokens.primary).toBeDefined();
    expect(tokens["on-primary"]).toBeDefined();
    expect(tokens.surface).toBeDefined();
    expect(tokens["on-surface"]).toBeDefined();
    expect(tokens.error).toBeDefined();
  });

  it("returns hex color values", () => {
    const tokens = generateMD3Tokens("#6750A4", false);
    expect(tokens.primary).toMatch(/^#[0-9a-f]{6}$/i);
  });
});

describe("applyMD3TokensToElement", () => {
  let el: HTMLElement;

  beforeEach(() => {
    el = document.createElement("div");
  });

  it("sets --wr-* CSS custom properties on the element", () => {
    applyMD3TokensToElement(el, "#6750A4", false);
    expect(el.style.getPropertyValue("--wr-primary")).toBeTruthy();
    expect(el.style.getPropertyValue("--wr-on-primary")).toBeTruthy();
    expect(el.style.getPropertyValue("--wr-surface")).toBeTruthy();
  });

  it("sets different values for dark mode", () => {
    const lightEl = document.createElement("div");
    const darkEl = document.createElement("div");
    applyMD3TokensToElement(lightEl, "#6750A4", false);
    applyMD3TokensToElement(darkEl, "#6750A4", true);
    expect(lightEl.style.getPropertyValue("--wr-primary")).not.toBe(
      darkEl.style.getPropertyValue("--wr-primary"),
    );
  });
});

describe("clearMD3TokensFromElement", () => {
  it("removes all --wr-* properties set by applyMD3TokensToElement", () => {
    const el = document.createElement("div");
    applyMD3TokensToElement(el, "#6750A4", false);
    expect(el.style.getPropertyValue("--wr-primary")).toBeTruthy();

    clearMD3TokensFromElement(el);
    expect(el.style.getPropertyValue("--wr-primary")).toBe("");
    expect(el.style.getPropertyValue("--wr-on-primary")).toBe("");
    expect(el.style.getPropertyValue("--wr-surface")).toBe("");
  });
});
