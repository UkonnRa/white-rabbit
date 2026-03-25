import {
  argbFromHex,
  hexFromArgb,
  Hct,
  MaterialDynamicColors,
  SchemeTonalSpot,
  type DynamicColor,
  type DynamicScheme,
} from "@material/material-color-utilities";
import { useLocalStorage } from "@vueuse/core";
import { useTheme } from "vuetify";

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

export const DEFAULT_SEED = PALETTE[0].hex;

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

export function useAppTheme() {
  const theme = useTheme();
  const seed = useLocalStorage("app-seed-color", DEFAULT_SEED);

  function applySeed(hex: string) {
    seed.value = hex;
    if (theme.themes.value.light) {
      Object.assign(
        theme.themes.value.light.colors,
        buildTheme(hex, false).colors,
      );
    }
    if (theme.themes.value.dark) {
      Object.assign(
        theme.themes.value.dark.colors,
        buildTheme(hex, true).colors,
      );
    }
  }

  return { seed, applySeed, PALETTE };
}
