import { describe, it, expect } from "vitest";
import { resolveRecipe } from "../../../composables/useRecipe";
import { inputRecipe } from "./input";
import { textareaRecipe } from "./textarea";
import { selectRecipe } from "./select";
import { comboboxRecipe } from "./combobox";
import { cardRecipe } from "./card";
import { chipRecipe } from "./chip";
import { dialogRecipe } from "./dialog";
import { menuRecipe } from "./menu";
import { tooltipRecipe } from "./tooltip";
import { switchRecipe } from "./switch";
import { iconRecipe } from "./icon";
import { dataTableRecipe } from "./dataTable";

describe("input recipe", () => {
  it("base includes border and bg-surface", () => {
    const classes = resolveRecipe(inputRecipe, {});
    expect(classes).toContain("rounded-md");
    expect(classes).toContain("border");
    expect(classes).toContain("bg-surface");
    expect(classes).toContain("text-on-surface");
  });

  it("error variant adds border-error", () => {
    const classes = resolveRecipe(inputRecipe, { variant: "error" });
    expect(classes).toContain("border-error");
  });

  it("md size has h-10", () => {
    const classes = resolveRecipe(inputRecipe, { size: "md" });
    expect(classes).toContain("h-10");
    expect(classes).toContain("px-3");
  });

  it("disabled replaces interactions", () => {
    const classes = resolveRecipe(inputRecipe, { disabled: true });
    expect(classes).toContain("opacity-50");
    expect(classes).toContain("cursor-not-allowed");
    expect(classes).not.toContain("hover:border-outline");
  });
});

describe("textarea recipe", () => {
  it("base includes resize-y", () => {
    const classes = resolveRecipe(textareaRecipe, {});
    expect(classes).toContain("resize-y");
    expect(classes).toContain("bg-surface");
  });

  it("lg size has larger padding", () => {
    const classes = resolveRecipe(textareaRecipe, { size: "lg" });
    expect(classes).toContain("px-4");
    expect(classes).toContain("py-3");
  });
});

describe("select recipe", () => {
  it("trigger variant has border and cursor-pointer", () => {
    const classes = resolveRecipe(selectRecipe, { variant: "trigger" });
    expect(classes).toContain("border");
    expect(classes).toContain("cursor-pointer");
  });

  it("content variant has shadow-lg", () => {
    const classes = resolveRecipe(selectRecipe, { variant: "content" });
    expect(classes).toContain("shadow-lg");
    expect(classes).toContain("rounded-md");
  });

  it("item variant has data-highlighted styles", () => {
    const classes = resolveRecipe(selectRecipe, { variant: "item" });
    expect(classes).toContain("cursor-pointer");
    expect(classes).toContain("select-none");
  });
});

describe("combobox recipe", () => {
  it("input variant has border and placeholder", () => {
    const classes = resolveRecipe(comboboxRecipe, { variant: "input" });
    expect(classes).toContain("border");
    expect(classes).toContain("placeholder:text-on-surface-variant");
  });

  it("item variant has data-highlighted styling", () => {
    const classes = resolveRecipe(comboboxRecipe, { variant: "item" });
    expect(classes).toContain("data-[highlighted]:bg-surface-variant");
  });

  it("empty variant is centered text", () => {
    const classes = resolveRecipe(comboboxRecipe, { variant: "empty" });
    expect(classes).toContain("text-center");
  });
});

describe("card recipe", () => {
  it("elevated variant has shadow", () => {
    const classes = resolveRecipe(cardRecipe, { variant: "elevated" });
    expect(classes).toContain("shadow-md");
  });

  it("outlined variant has border", () => {
    const classes = resolveRecipe(cardRecipe, { variant: "outlined" });
    expect(classes).toContain("border");
    expect(classes).toContain("border-outline-variant");
  });

  it("filled variant has bg-surface-variant", () => {
    const classes = resolveRecipe(cardRecipe, { variant: "filled" });
    expect(classes).toContain("bg-surface-variant");
  });

  it("md size has p-4", () => {
    const classes = resolveRecipe(cardRecipe, { size: "md" });
    expect(classes).toContain("p-4");
  });
});

describe("chip recipe", () => {
  it("tonal variant has bg-surface-variant", () => {
    const classes = resolveRecipe(chipRecipe, { variant: "tonal" });
    expect(classes).toContain("bg-surface-variant");
    expect(classes).toContain("text-on-surface-variant");
  });

  it("solid variant has bg-primary", () => {
    const classes = resolveRecipe(chipRecipe, { variant: "solid" });
    expect(classes).toContain("bg-primary");
  });

  it("sm size has h-6", () => {
    const classes = resolveRecipe(chipRecipe, { size: "sm" });
    expect(classes).toContain("h-6");
    expect(classes).toContain("text-xs");
  });
});

describe("dialog recipe", () => {
  it("overlay variant has fixed positioning", () => {
    const classes = resolveRecipe(dialogRecipe, { variant: "overlay" });
    expect(classes).toContain("fixed");
    expect(classes).toContain("inset-0");
    expect(classes).toContain("bg-black/50");
  });

  it("content variant has centered positioning and shadow", () => {
    const classes = resolveRecipe(dialogRecipe, { variant: "content" });
    expect(classes).toContain("fixed");
    expect(classes).toContain("shadow-xl");
    expect(classes).toContain("rounded-lg");
  });

  it("content with md size has max-w-lg", () => {
    const classes = resolveRecipe(dialogRecipe, {
      variant: "content",
      size: "md",
    });
    expect(classes).toContain("max-w-lg");
  });

  it("title variant has font-semibold", () => {
    const classes = resolveRecipe(dialogRecipe, { variant: "title" });
    expect(classes).toContain("font-semibold");
  });
});

describe("menu recipe", () => {
  it("content variant has shadow and border", () => {
    const classes = resolveRecipe(menuRecipe, { variant: "content" });
    expect(classes).toContain("shadow-lg");
    expect(classes).toContain("border");
    expect(classes).toContain("rounded-md");
  });

  it("item variant has data-highlighted", () => {
    const classes = resolveRecipe(menuRecipe, { variant: "item" });
    expect(classes).toContain("data-[highlighted]:bg-surface-variant");
  });

  it("separator variant has h-px", () => {
    const classes = resolveRecipe(menuRecipe, { variant: "separator" });
    expect(classes).toContain("h-px");
    expect(classes).toContain("bg-outline-variant");
  });
});

describe("tooltip recipe", () => {
  it("content variant has inverted colors and animation", () => {
    const classes = resolveRecipe(tooltipRecipe, { variant: "content" });
    expect(classes).toContain("bg-on-surface");
    expect(classes).toContain("text-surface");
    expect(classes).toContain("rounded-md");
  });
});

describe("switch recipe", () => {
  it("root variant has transition and checked state", () => {
    const classes = resolveRecipe(switchRecipe, { variant: "root" });
    expect(classes).toContain("rounded-full");
    expect(classes).toContain("data-[state=checked]:bg-primary");
  });

  it("thumb variant has shadow and transition", () => {
    const classes = resolveRecipe(switchRecipe, { variant: "thumb" });
    expect(classes).toContain("rounded-full");
    expect(classes).toContain("shadow-sm");
  });

  it("md size has h-6 w-11", () => {
    const classes = resolveRecipe(switchRecipe, { size: "md" });
    expect(classes).toContain("h-6");
    expect(classes).toContain("w-11");
  });
});

describe("icon recipe", () => {
  it("base includes inline-flex", () => {
    const classes = resolveRecipe(iconRecipe, {});
    expect(classes).toContain("inline-flex");
    expect(classes).toContain("shrink-0");
  });

  it("md size has size-5", () => {
    const classes = resolveRecipe(iconRecipe, { size: "md" });
    expect(classes).toContain("size-5");
  });

  it("xl size has size-8", () => {
    const classes = resolveRecipe(iconRecipe, { size: "xl" });
    expect(classes).toContain("size-8");
  });
});

describe("dataTable recipe", () => {
  it("table variant has w-full", () => {
    const classes = resolveRecipe(dataTableRecipe, { variant: "table" });
    expect(classes).toContain("w-full");
    expect(classes).toContain("text-sm");
  });

  it("headerCell variant has font-medium", () => {
    const classes = resolveRecipe(dataTableRecipe, { variant: "headerCell" });
    expect(classes).toContain("font-medium");
    expect(classes).toContain("text-on-surface-variant");
  });

  it("row variant has border and hover", () => {
    const classes = resolveRecipe(dataTableRecipe, { variant: "row" });
    expect(classes).toContain("border-b");
    expect(classes).toContain("hover:bg-surface-variant/50");
  });

  it("cell variant has px-3 py-2", () => {
    const classes = resolveRecipe(dataTableRecipe, { variant: "cell" });
    expect(classes).toContain("px-3");
    expect(classes).toContain("py-2");
  });

  it("empty variant is centered", () => {
    const classes = resolveRecipe(dataTableRecipe, { variant: "empty" });
    expect(classes).toContain("text-center");
    expect(classes).toContain("text-on-surface-variant");
  });
});
