import type { ComputedRef, Directive, InjectionKey } from "vue";

// ── Token schemas ──────────────────────────────────────────────────────────

/** Minimum token set every theme must provide. */
export interface BaseTokens {
  color: {
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
  };
  radius: {
    sm: string;
    md: string;
    lg: string;
    xl: string;
    full: string;
  };
  spacing: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
  motion: {
    durationFast: string;
    durationNormal: string;
    durationSlow: string;
    easingStandard: string;
    easingEmphasized: string;
  };
}

/** MD3 extends base tokens with tertiary palette, state-layer, elevation. */
export interface MD3Tokens extends BaseTokens {
  color: BaseTokens["color"] & {
    tertiary: string;
    onTertiary: string;
    tertiaryContainer: string;
    primaryContainer: string;
    onPrimaryContainer: string;
    surfaceContainerLowest: string;
    surfaceContainerLow: string;
    surfaceContainer: string;
    surfaceContainerHigh: string;
    surfaceContainerHighest: string;
    inverseSurface: string;
    inverseOnSurface: string;
    inversePrimary: string;
  };
  stateLayer: {
    hoverOpacity: string;
    focusOpacity: string;
    pressedOpacity: string;
    draggedOpacity: string;
  };
  elevation: {
    level0: string;
    level1: string;
    level2: string;
    level3: string;
    level4: string;
    level5: string;
  };
}

// ── Recipe system ──────────────────────────────────────────────────────────

/** Style recipe for a single component within a theme. */
export interface ComponentRecipe {
  /** Always-applied classes. */
  base: string[];
  /** Variant name -> class list. */
  variants: Record<string, string[]>;
  /** Size name -> class list. */
  sizes: Record<string, string[]>;
  /** Interaction state -> class list. */
  interactions: {
    hover?: string[];
    focus?: string[];
    pressed?: string[];
    dragged?: string[];
    disabled?: string[];
  };
  /** Optional compound variants (variant + size overrides). */
  compound?: Array<{
    variant?: string;
    size?: string;
    classes: string[];
  }>;
}

// ── Theme definition ───────────────────────────────────────────────────────

/** Recipe map: one entry per base component. */
export interface ThemeRecipes {
  button?: ComponentRecipe;
  input?: ComponentRecipe;
  textarea?: ComponentRecipe;
  select?: ComponentRecipe;
  combobox?: ComponentRecipe;
  card?: ComponentRecipe;
  chip?: ComponentRecipe;
  dialog?: ComponentRecipe;
  menu?: ComponentRecipe;
  tooltip?: ComponentRecipe;
  switch?: ComponentRecipe;
  dataTable?: ComponentRecipe;
  [key: string]: ComponentRecipe | undefined;
}

/** Complete theme: recipes + optional directives + optional stylesheets. */
export interface ThemeDefinition {
  name: string;
  recipes: ThemeRecipes;
  /** Vue directives provided by this theme (e.g. ripple). */
  directives?: Record<string, Directive>;
  /** CSS file paths to import when this theme is active. */
  stylesheets?: string[];
}

// ── Provide/inject keys ────────────────────────────────────────────────────

export const THEME_KEY = Symbol("wr-theme") as InjectionKey<
  ComputedRef<ThemeDefinition>
>;
export const MODE_KEY = Symbol("wr-mode") as InjectionKey<
  ComputedRef<"light" | "dark">
>;
