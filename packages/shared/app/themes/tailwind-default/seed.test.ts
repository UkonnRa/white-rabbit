import { describe, it, expect } from "vitest";
import {
  hueFromHex,
  generateTailwindTokens,
  applyTailwindTokensToElement,
  clearTailwindTokensFromElement,
} from "./seed";

describe("hueFromHex", () => {
  it("extracts ~0 for red", () => {
    const hue = hueFromHex("#ff0000");
    expect(hue).toBe(0);
  });

  it("extracts ~120 for green", () => {
    const hue = hueFromHex("#00ff00");
    expect(hue).toBe(120);
  });

  it("extracts ~240 for blue", () => {
    const hue = hueFromHex("#0000ff");
    expect(hue).toBe(240);
  });

  it("returns 260 for achromatic gray", () => {
    const hue = hueFromHex("#808080");
    expect(hue).toBe(260);
  });
});

describe("generateTailwindTokens", () => {
  it("generates light mode tokens with oklch values", () => {
    const tokens = generateTailwindTokens("#3949AB", false);
    expect(tokens.primary).toMatch(/^oklch\(/);
    expect(tokens.surface).toMatch(/^oklch\(/);
  });

  it("generates dark mode tokens with different lightness", () => {
    const light = generateTailwindTokens("#3949AB", false);
    const dark = generateTailwindTokens("#3949AB", true);
    // Dark primary should be lighter than light primary (shade 400 vs 600)
    expect(light.primary).not.toBe(dark.primary);
    expect(light.surface).not.toBe(dark.surface);
  });

  it("produces different primary for different seeds", () => {
    const blue = generateTailwindTokens("#0000ff", false);
    const red = generateTailwindTokens("#ff0000", false);
    expect(blue.primary).not.toBe(red.primary);
  });

  it("keeps surfaces the same regardless of seed", () => {
    const blue = generateTailwindTokens("#0000ff", false);
    const red = generateTailwindTokens("#ff0000", false);
    // Surfaces are on the neutral scale — independent of accent hue
    expect(blue.surface).toBe(red.surface);
    expect(blue.background).toBe(red.background);
    expect(blue.surfaceVariant).toBe(red.surfaceVariant);
  });

  it("keeps error constant regardless of seed", () => {
    const a = generateTailwindTokens("#3949AB", false);
    const b = generateTailwindTokens("#00695C", false);
    expect(a.error).toBe(b.error);
  });
});

describe("applyTailwindTokensToElement", () => {
  it("sets --wr-primary on element", () => {
    const el = document.createElement("div");
    applyTailwindTokensToElement(el, "#3949AB", false);
    expect(el.style.getPropertyValue("--wr-primary")).toMatch(/^oklch\(/);
  });

  it("sets --wr-surface on element", () => {
    const el = document.createElement("div");
    applyTailwindTokensToElement(el, "#3949AB", false);
    expect(el.style.getPropertyValue("--wr-surface")).toMatch(/^oklch\(/);
  });
});

describe("clearTailwindTokensFromElement", () => {
  it("removes all --wr-* properties", () => {
    const el = document.createElement("div");
    applyTailwindTokensToElement(el, "#3949AB", false);
    expect(el.style.getPropertyValue("--wr-primary")).toBeTruthy();

    clearTailwindTokensFromElement(el);
    expect(el.style.getPropertyValue("--wr-primary")).toBe("");
    expect(el.style.getPropertyValue("--wr-surface")).toBe("");
  });
});
