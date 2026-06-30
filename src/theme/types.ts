/** Mirrors mobile `AccentPalette` in theme_presets.dart */
export type AccentPalette =
  | "slate"
  | "green"
  | "purple"
  | "amber"
  | "blue"
  | "rose"
  | "cyan"
  | "obsidian"
  | "linen";

export type ThemeMode = "dark" | "light" | "system";

export interface SurfaceColors {
  scaffold: string;
  surface: string;
  card: string;
  elevated: string;
}

export interface TextColors {
  primary: string;
  secondary: string;
  tertiary: string;
}

export interface BorderColors {
  card: string;
  subtle: string;
}

export interface AccentColors {
  accent: string;
  success: string;
  danger: string;
  warning: string;
  info: string;
}

export interface GlassColors {
  fill: string;
  border: string;
  fillOpacity: number;
  borderOpacity: number;
  blurSigma: number;
}

export interface ProgressColors {
  track: string;
  low: string;
  mid: string;
  high: string;
  glow: string;
  tickMark: string;
  tickLabel: string;
}

export interface MetricColors {
  bar1: string;
  bar2: string;
  bar3: string;
  bar4: string;
  empty: string;
}

export interface StatusColors {
  clear: string;
  active: string;
  dot: string;
}

export interface InteractiveColors {
  buttonFill: string;
  buttonBorder: string;
  inputBorder: string;
  inputBorderFocused: string;
}

export interface GridColors {
  cell: string;
  cellHighlight: string;
  outline: string;
}

export interface CoreColors {
  surfaces: SurfaceColors;
  text: TextColors;
  borders: BorderColors;
  accents: AccentColors;
  glass: GlassColors;
  progress: ProgressColors;
  metrics: MetricColors;
  status: StatusColors;
  interactive: InteractiveColors;
  grid: GridColors;
}

export interface UiPrefs {
  theme_mode: ThemeMode;
  accent_palette: AccentPalette;
  layout_locked: boolean;
}

export const ACCENT_PALETTES: AccentPalette[] = [
  "slate",
  "green",
  "purple",
  "amber",
  "blue",
  "rose",
  "cyan",
  "obsidian",
  "linen",
];

export const ACCENT_LABELS: Record<AccentPalette, string> = {
  slate: "Slate",
  green: "Green",
  purple: "Purple",
  amber: "Amber",
  blue: "Blue",
  rose: "Rose",
  cyan: "Cyan",
  obsidian: "Obsidian",
  linen: "Linen",
};
