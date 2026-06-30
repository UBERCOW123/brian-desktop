/** Convert mobile AARRGGBB (#AARRGGBB) or RRGGBB to CSS color. */
export function cssColor(hex: string): string {
  const raw = hex.startsWith("#") ? hex.slice(1) : hex;
  if (raw.length === 8) {
    const a = parseInt(raw.slice(0, 2), 16) / 255;
    const r = parseInt(raw.slice(2, 4), 16);
    const g = parseInt(raw.slice(4, 6), 16);
    const b = parseInt(raw.slice(6, 8), 16);
    if (a >= 0.999) return `#${raw.slice(2)}`;
    return `rgba(${r}, ${g}, ${b}, ${Number(a.toFixed(3))})`;
  }
  if (raw.length === 6) return `#${raw}`;
  return hex;
}

/** Parse widget configJson color slot (#AARRGGBB). */
export function parseWidgetColorSlot(raw: string | undefined): string | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (!/^#[0-9A-Fa-f]{6,8}$/.test(trimmed)) return null;
  return cssColor(trimmed.length === 7 ? `#FF${trimmed.slice(1)}` : trimmed);
}
