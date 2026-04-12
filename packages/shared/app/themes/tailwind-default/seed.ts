/**
 * Tailwind-default seed color system.
 *
 * Unlike MD3's single-seed-derives-everything approach, tailwind-default
 * treats the seed as an **accent hue picker**. The seed hex is converted to
 * a hue angle, which generates an oklch primary scale. Surfaces stay on a
 * fixed neutral scale (no primary tint). Secondary maps to a neutral mid-tone.
 *
 * This mirrors how Tailwind / shadcn / Radix handle color: the designer
 * picks an accent scale independently from the neutral ramp.
 *
 * See docs/architecture/color-palette-design.md for the rationale.
 */

// ── oklch scale generation ─────────────────────────────────────────────────

interface OklchStep {
  lightness: number;
  chroma: number;
}

/**
 * Shade stops for the primary accent scale.
 * Lightness and chroma tuned for oklch to produce a visually even ramp.
 */
const ACCENT_SCALE: Record<number, OklchStep> = {
  50: { lightness: 0.97, chroma: 0.02 },
  100: { lightness: 0.93, chroma: 0.04 },
  200: { lightness: 0.87, chroma: 0.08 },
  300: { lightness: 0.78, chroma: 0.12 },
  400: { lightness: 0.68, chroma: 0.16 },
  500: { lightness: 0.58, chroma: 0.18 },
  600: { lightness: 0.5, chroma: 0.2 },
  700: { lightness: 0.42, chroma: 0.18 },
  800: { lightness: 0.34, chroma: 0.14 },
  900: { lightness: 0.27, chroma: 0.1 },
  950: { lightness: 0.2, chroma: 0.06 },
};

/**
 * Fixed neutral scale for surfaces (slate-like, hue 260, minimal chroma).
 */
const NEUTRAL_SCALE: Record<number, OklchStep> = {
  25: { lightness: 0.985, chroma: 0.003 },
  50: { lightness: 0.97, chroma: 0.005 },
  100: { lightness: 0.93, chroma: 0.008 },
  200: { lightness: 0.87, chroma: 0.01 },
  300: { lightness: 0.78, chroma: 0.012 },
  400: { lightness: 0.68, chroma: 0.015 },
  500: { lightness: 0.55, chroma: 0.02 },
  600: { lightness: 0.45, chroma: 0.02 },
  700: { lightness: 0.37, chroma: 0.02 },
  800: { lightness: 0.28, chroma: 0.015 },
  900: { lightness: 0.2, chroma: 0.012 },
  950: { lightness: 0.16, chroma: 0.01 },
};

const NEUTRAL_HUE = 260;

function oklch(l: number, c: number, h: number): string {
  return `oklch(${l} ${c} ${h})`;
}

// ── Hex → hue extraction ───────────────────────────────────────────────────

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  return [
    parseInt(h.slice(0, 2), 16) / 255,
    parseInt(h.slice(2, 4), 16) / 255,
    parseInt(h.slice(4, 6), 16) / 255,
  ];
}

/**
 * Extract an approximate hue angle (0–360) from a hex color.
 * Uses simple RGB→HSL hue extraction — we only need the hue, not full
 * perceptual accuracy, since it feeds into oklch scale generation.
 */
export function hueFromHex(hex: string): number {
  const [r, g, b] = hexToRgb(hex);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;

  if (d === 0) return 260; // achromatic → default blue-ish hue

  let h: number;
  if (max === r) h = ((g - b) / d) % 6;
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;

  h = Math.round(h * 60);
  if (h < 0) h += 360;
  return h;
}

// ── Token generation ───────────────────────────────────────────────────────

export interface TailwindSeedTokens {
  primary: string;
  onPrimary: string;
  secondary: string;
  onSecondary: string;
  surface: string;
  onSurface: string;
  surfaceVariant: string;
  onSurfaceVariant: string;
  background: string;
  onBackground: string;
  error: string;
  onError: string;
  outline: string;
  outlineVariant: string;
}

/**
 * Generate tailwind-default tokens from a seed hex.
 *
 * The seed determines the accent hue. Surfaces stay on a fixed neutral scale.
 * Dark mode flips which end of the scale is used (shade inversion).
 */
export function generateTailwindTokens(
  seedHex: string,
  dark: boolean,
): TailwindSeedTokens {
  const hue = hueFromHex(seedHex);

  function accent(shade: number): string {
    const s = ACCENT_SCALE[shade];
    return oklch(s.lightness, s.chroma, hue);
  }

  function neutral(shade: number): string {
    const s = NEUTRAL_SCALE[shade];
    return oklch(s.lightness, s.chroma, NEUTRAL_HUE);
  }

  if (dark) {
    return {
      primary: accent(400),
      onPrimary: accent(950),
      secondary: neutral(400),
      onSecondary: neutral(950),
      surface: neutral(900),
      onSurface: neutral(100),
      surfaceVariant: neutral(800),
      onSurfaceVariant: neutral(300),
      background: neutral(950),
      onBackground: neutral(100),
      error: oklch(0.75, 0.18, 27),
      onError: oklch(0.2, 0.08, 27),
      outline: neutral(500),
      outlineVariant: neutral(700),
    };
  }

  return {
    primary: accent(600),
    onPrimary: oklch(1, 0, 0),
    secondary: neutral(600),
    onSecondary: oklch(1, 0, 0),
    surface: neutral(50),
    onSurface: neutral(900),
    surfaceVariant: neutral(200),
    onSurfaceVariant: neutral(600),
    background: neutral(25),
    onBackground: neutral(900),
    error: oklch(0.55, 0.22, 27),
    onError: oklch(1, 0, 0),
    outline: neutral(500),
    outlineVariant: neutral(300),
  };
}

// ── Token name mapping ─────────────────────────────────────────────────────

const TOKEN_MAP: Record<keyof TailwindSeedTokens, string> = {
  primary: "primary",
  onPrimary: "on-primary",
  secondary: "secondary",
  onSecondary: "on-secondary",
  surface: "surface",
  onSurface: "on-surface",
  surfaceVariant: "surface-variant",
  onSurfaceVariant: "on-surface-variant",
  background: "background",
  onBackground: "on-background",
  error: "error",
  onError: "on-error",
  outline: "outline",
  outlineVariant: "outline-variant",
};

/**
 * Apply tailwind-default seed tokens as inline CSS vars on an element.
 */
export function applyTailwindTokensToElement(
  el: HTMLElement,
  seedHex: string,
  dark: boolean,
): void {
  const tokens = generateTailwindTokens(seedHex, dark);

  for (const [key, wrName] of Object.entries(TOKEN_MAP)) {
    const value = tokens[key as keyof TailwindSeedTokens];
    el.style.setProperty(`--wr-${wrName}`, value);
  }
}

/**
 * Remove all tailwind-default inline token overrides from an element.
 */
export function clearTailwindTokensFromElement(el: HTMLElement): void {
  for (const wrName of Object.values(TOKEN_MAP)) {
    el.style.removeProperty(`--wr-${wrName}`);
  }
}
