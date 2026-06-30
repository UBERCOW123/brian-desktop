import { invoke } from "@tauri-apps/api/core";
import type { CoreColors, UiPrefs } from "./types";
import { isDarkMode, resolvePreset } from "./presets";

const DEFAULT_PREFS: UiPrefs = {
  theme_mode: "dark",
  accent_palette: "green",
  layout_locked: false,
};

let currentPrefs: UiPrefs = { ...DEFAULT_PREFS };
let currentColors: CoreColors = resolvePreset("dark", "green");

export function getColors(): CoreColors {
  return currentColors;
}

export function getUiPrefs(): UiPrefs {
  return currentPrefs;
}

export function applyTheme(prefs: UiPrefs): void {
  currentPrefs = prefs;
  currentColors = resolvePreset(prefs.theme_mode, prefs.accent_palette);
  const root = document.documentElement;
  const dark = isDarkMode(prefs.theme_mode);
  root.dataset.theme = dark ? "dark" : "light";
  root.dataset.accent = prefs.accent_palette;
  root.style.colorScheme = dark ? "dark" : "light";
  root.classList.toggle("dark", dark);

  const c = currentColors;
  const set = (name: string, value: string | number) => {
    root.style.setProperty(name, String(value));
  };

  set("--surface-scaffold", c.surfaces.scaffold);
  set("--surface-surface", c.surfaces.surface);
  set("--surface-card", c.surfaces.card);
  set("--surface-elevated", c.surfaces.elevated);
  set("--text-primary", c.text.primary);
  set("--text-secondary", c.text.secondary);
  set("--text-tertiary", c.text.tertiary);
  set("--border-card", c.borders.card);
  set("--border-subtle", c.borders.subtle);
  set("--accent", c.accents.accent);
  set("--accent-success", c.accents.success);
  set("--accent-danger", c.accents.danger);
  set("--accent-warning", c.accents.warning);
  set("--accent-info", c.accents.info);
  set("--glass-fill", c.glass.fill);
  set("--glass-border", c.glass.border);
  set("--glass-fill-opacity", c.glass.fillOpacity);
  set("--glass-border-opacity", c.glass.borderOpacity);
  set("--glass-blur", `${c.glass.blurSigma}px`);
  set("--interactive-button-fill", c.interactive.buttonFill);
  set("--interactive-button-border", c.interactive.buttonBorder);
  set("--interactive-input-border", c.interactive.inputBorder);
  set("--interactive-input-focus", c.interactive.inputBorderFocused);
  set("--grid-cell", c.grid.cell);
  set("--grid-cell-highlight", c.grid.cellHighlight);
  set("--grid-outline", c.grid.outline);
  set("--status-active", c.status.active);
  window.dispatchEvent(new CustomEvent("theme:changed"));
}

export async function loadThemeFromBackend(): Promise<UiPrefs> {
  try {
    const prefs = await invoke<UiPrefs>("get_ui_prefs");
    applyTheme(prefs);
    return prefs;
  } catch {
    applyTheme(DEFAULT_PREFS);
    return DEFAULT_PREFS;
  }
}

export async function saveThemePref(
  patch: Partial<Pick<UiPrefs, "theme_mode" | "accent_palette" | "layout_locked">>,
): Promise<UiPrefs> {
  const next = { ...currentPrefs, ...patch };
  await invoke("set_ui_prefs", { prefs: next });
  applyTheme(next);
  return next;
}

export async function loadShellLayout(): Promise<string | null> {
  try {
    return await invoke<string | null>("get_shell_layout");
  } catch {
    return null;
  }
}

export async function saveShellLayout(layoutJson: string): Promise<void> {
  await invoke("set_shell_layout", { layout: layoutJson });
}

export async function loadAssistPrefs<T>(): Promise<T | null> {
  try {
    const raw = await invoke<string | null>("get_assist_prefs");
    if (!raw) return null;
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function saveAssistPrefs<T>(prefs: T): Promise<void> {
  await invoke("set_assist_prefs", { prefs: JSON.stringify(prefs) });
}
