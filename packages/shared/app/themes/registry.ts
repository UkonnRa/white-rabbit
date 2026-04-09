import type { ThemeDefinition } from "./types";

const themes = new Map<string, ThemeDefinition>();

export function registerTheme(theme: ThemeDefinition): void {
  themes.set(theme.name, theme);
}

export function getTheme(name: string): ThemeDefinition {
  const theme = themes.get(name);
  if (!theme) {
    throw new Error(
      `Theme "${name}" not registered. Available: ${[...themes.keys()].join(", ") || "(none)"}`,
    );
  }
  return theme;
}

export function listThemes(): string[] {
  return [...themes.keys()];
}

/** Remove all registered themes. Useful for testing. */
export function clearThemes(): void {
  themes.clear();
}
