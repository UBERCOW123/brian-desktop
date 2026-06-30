export type GlassDepth = "surface" | "elevated" | "solid";

const DEPTH_CLASS: Record<GlassDepth, string> = {
  solid: "brian-surface",
  surface: "brian-glass brian-glass--surface",
  elevated: "brian-glass brian-glass--elevated",
};

export interface GlassCardOptions {
  depth?: GlassDepth;
  padding?: string;
  borderRadius?: string;
  borderColor?: string;
  className?: string;
  onClick?: () => void;
}

export function createGlassCard(content: HTMLElement | string, options: GlassCardOptions = {}): HTMLElement {
  const depth = options.depth ?? "elevated";
  const card = document.createElement("div");
  card.className = `glass-card ${DEPTH_CLASS[depth]} ${options.className ?? ""}`.trim();
  if (options.padding) card.style.setProperty("--glass-inner-padding", options.padding);
  if (options.borderRadius) card.style.borderRadius = options.borderRadius;
  if (options.borderColor) card.style.setProperty("--glass-border-override", options.borderColor);

  const inner = document.createElement("div");
  inner.className = "glass-card__inner";
  if (typeof content === "string") {
    inner.innerHTML = content;
  } else {
    inner.appendChild(content);
  }
  card.appendChild(inner);

  if (options.onClick) {
    card.classList.add("glass-card--interactive");
    card.addEventListener("click", options.onClick);
  }
  return card;
}
