import {
  argbFromHex,
  hexFromArgb,
  Hct,
  MaterialDynamicColors,
  SchemeTonalSpot,
  type DynamicColor,
  type DynamicScheme,
} from "@material/material-color-utilities";
import { inject } from "vue";
import { THEME_NAME_KEY, SEED_COLOR_KEY } from "../plugins/theme";

// ── Safe colors: excludes red/orange/yellow to avoid conflict with
//    error (red) and warning (yellow/amber) semantic colors ──────────────────

export const PALETTE = [
  { label: "Golden Sand", hex: "#eccc68" },
  { label: "Peace", hex: "#a4b0be" },
  { label: "Purple", hex: "#6750A4" },
  { label: "Prestige Blue", hex: "#2f3542" },
  { label: "Indigo", hex: "#3949AB" },
  { label: "Navy Blue", hex: "#1A237E" },
  { label: "Blue", hex: "#1565C0" },
  { label: "Orange", hex: "#ff6348" },
  { label: "Teal", hex: "#00695C" },
  { label: "Cyan", hex: "#00838F" },
  { label: "Blue Grey", hex: "#455A64" },
  { label: "Brown", hex: "#4E342E" },
] as const;

export const DEFAULT_SEED = "#6750A4";

export const AVAILABLE_THEMES = [
  { name: "tailwind-default", label: "Tailwind Default" },
  { name: "md3-expressive", label: "MD3 Expressive" },
] as const;

// ── Color generation ────────────────────────────────────────────────────────

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

export function buildTheme(hex: string, dark: boolean) {
  const source = Hct.fromInt(argbFromHex(hex));
  const scheme = new SchemeTonalSpot(source, dark, 0);
  return { dark, colors: schemeToColors(scheme) };
}

// ── Composable ──────────────────────────────────────────────────────────────

/**
 * Theme and seed color management composable.
 *
 * Injects the reactive theme name and seed color refs provided by the
 * Nuxt plugin. Components use this to switch themes and change the
 * MD3 seed color.
 */
export function useAppTheme() {
  const themeName = inject(THEME_NAME_KEY);
  const seedColor = inject(SEED_COLOR_KEY);

  if (!themeName || !seedColor) {
    throw new Error(
      "useAppTheme: no theme context found. Is the theme plugin installed?",
    );
  }

  function setTheme(name: string) {
    themeName.value = name;
  }

  function setSeedColor(hex: string) {
    seedColor.value = hex;
  }

  return {
    themeName,
    seedColor,
    setTheme,
    setSeedColor,
    PALETTE,
    AVAILABLE_THEMES,
  };
}
