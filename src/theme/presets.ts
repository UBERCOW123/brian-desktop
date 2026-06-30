import rawPresets from "./generated/presets.json";
import type { AccentPalette, CoreColors, ThemeMode } from "./types";
import { cssColor } from "./color-utils";

type RawGroup = { colors: string[]; nums: number[] };
type RawPreset = Record<string, RawGroup>;
const generated = rawPresets as Record<string, RawPreset>;

function buildColors(raw: RawPreset): CoreColors {
  const s = raw.surfaces.colors;
  const t = raw.text.colors;
  const b = raw.borders.colors;
  const a = raw.accents.colors;
  const g = raw.glass;
  const p = raw.progress.colors;
  const m = raw.metrics.colors;
  const st = raw.status.colors;
  const i = raw.interactive.colors;
  const gr = raw.grid.colors;

  return {
    surfaces: {
      scaffold: cssColor(s[0]),
      surface: cssColor(s[1]),
      card: cssColor(s[2]),
      elevated: cssColor(s[3]),
    },
    text: {
      primary: cssColor(t[0]),
      secondary: cssColor(t[1]),
      tertiary: cssColor(t[2]),
    },
    borders: {
      card: cssColor(b[0]),
      subtle: cssColor(b[1]),
    },
    accents: {
      accent: cssColor(a[0]),
      success: cssColor(a[1]),
      danger: cssColor(a[2]),
      warning: cssColor(a[3]),
      info: cssColor(a[4]),
    },
    glass: {
      fill: cssColor(g.colors[0]),
      border: cssColor(g.colors[1]),
      fillOpacity: g.nums[0] ?? 0.06,
      borderOpacity: g.nums[1] ?? 0.08,
      blurSigma: g.nums[2] ?? 40,
    },
    progress: {
      track: cssColor(p[0]),
      low: cssColor(p[1]),
      mid: cssColor(p[2]),
      high: cssColor(p[3]),
      glow: cssColor(p[4]),
      tickMark: cssColor(p[5]),
      tickLabel: cssColor(p[6]),
    },
    metrics: {
      bar1: cssColor(m[0]),
      bar2: cssColor(m[1]),
      bar3: cssColor(m[2]),
      bar4: cssColor(m[3]),
      empty: cssColor(m[4]),
    },
    status: {
      clear: cssColor(st[0]),
      active: cssColor(st[1]),
      dot: cssColor(st[2]),
    },
    interactive: {
      buttonFill: cssColor(i[0]),
      buttonBorder: cssColor(i[1]),
      inputBorder: cssColor(i[2]),
      inputBorderFocused: cssColor(i[3]),
    },
    grid: {
      cell: cssColor(gr[0]),
      cellHighlight: cssColor(gr[1]),
      outline: cssColor(gr[2]),
    },
  };
}

const presetCache = new Map<string, CoreColors>();

function presetKey(mode: ThemeMode, accent: AccentPalette): string {
  const brightness =
    mode === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : mode;
  const accentName = accent.charAt(0).toUpperCase() + accent.slice(1);
  return `${brightness}${accentName}`;
}

export function resolvePreset(mode: ThemeMode, accent: AccentPalette): CoreColors {
  const key = presetKey(mode, accent);
  let cached = presetCache.get(key);
  if (!cached) {
    const raw = generated[key];
    if (!raw) {
      throw new Error(`Missing theme preset: ${key}`);
    }
    cached = buildColors(raw);
    presetCache.set(key, cached);
  }
  return cached;
}

export function isDarkMode(mode: ThemeMode): boolean {
  if (mode === "dark") return true;
  if (mode === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}
