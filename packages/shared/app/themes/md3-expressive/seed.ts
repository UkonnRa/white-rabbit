import {
  argbFromHex,
  hexFromArgb,
  Hct,
  MaterialDynamicColors,
  SchemeTonalSpot,
  type DynamicColor,
  type DynamicScheme,
} from "@material/material-color-utilities";

function toKebabCase(str: string) {
  return str.replaceAll(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
}

function schemeToColors(scheme: DynamicScheme): Record<string, string> {
  const colors: Record<string, string> = {};
  for (const [key, value] of Object.entries(MaterialDynamicColors)) {
    if (value && typeof (value as DynamicColor).getArgb === "function") {
      colors[toKebabCase(key)] = hexFromArgb(
        (value as DynamicColor).getArgb(scheme),
      );
    }
  }
  return colors;
}

/**
 * Generate a full set of MD3 color tokens from a seed hex value.
 *
 * Returns a flat Record mapping CSS variable names (without `--wr-` prefix)
 * to hex color strings.
 */
export function generateMD3Tokens(
  seedHex: string,
  dark: boolean,
): Record<string, string> {
  const source = Hct.fromInt(argbFromHex(seedHex));
  const scheme = new SchemeTonalSpot(source, dark, 0);
  return schemeToColors(scheme);
}

/**
 * Map of MD3 dynamic color keys to our `--wr-*` CSS variable names.
 *
 * This maps the kebab-case keys from MaterialDynamicColors to the
 * semantic variable names used by the theme system.
 */
const MD3_TO_WR_MAP: Record<string, string> = {
  primary: "primary",
  "on-primary": "on-primary",
  secondary: "secondary",
  "on-secondary": "on-secondary",
  tertiary: "tertiary",
  "on-tertiary": "on-tertiary",
  error: "error",
  "on-error": "on-error",
  surface: "surface",
  "on-surface": "on-surface",
  "surface-variant": "surface-variant",
  "on-surface-variant": "on-surface-variant",
  background: "background",
  "on-background": "on-background",
  outline: "outline",
  "outline-variant": "outline-variant",
  "primary-container": "primary-container",
  "on-primary-container": "on-primary-container",
  "tertiary-container": "tertiary-container",
  "inverse-surface": "inverse-surface",
  "inverse-on-surface": "inverse-on-surface",
  "inverse-primary": "inverse-primary",
  "surface-container-lowest": "surface-container-lowest",
  "surface-container-low": "surface-container-low",
  "surface-container": "surface-container",
  "surface-container-high": "surface-container-high",
  "surface-container-highest": "surface-container-highest",
};

/**
 * Apply seed-derived color tokens as CSS custom properties on a target element.
 *
 * Generates colors from the seed, then sets `--wr-<name>` on the element's
 * inline style for each mapped token. Works for any theme — the seed algorithm
 * produces a full harmonious palette that maps to `--wr-*` variables.
 */
export function applySeedTokensToElement(
  el: HTMLElement,
  seedHex: string,
  dark: boolean,
): void {
  const colors = generateMD3Tokens(seedHex, dark);

  for (const [md3Key, wrKey] of Object.entries(MD3_TO_WR_MAP)) {
    if (colors[md3Key]) {
      el.style.setProperty(`--wr-${wrKey}`, colors[md3Key]);
    }
  }
}

/**
 * Remove all seed-derived inline token overrides from an element.
 */
export function clearSeedTokensFromElement(el: HTMLElement): void {
  for (const wrKey of Object.values(MD3_TO_WR_MAP)) {
    el.style.removeProperty(`--wr-${wrKey}`);
  }
}

/** @deprecated Use applySeedTokensToElement */
export const applyMD3TokensToElement = applySeedTokensToElement;
/** @deprecated Use clearSeedTokensFromElement */
export const clearMD3TokensFromElement = clearSeedTokensFromElement;
