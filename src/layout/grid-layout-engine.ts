import type { GridMetrics } from "./grid-metrics";
import {
  gridMetricsFromWidth,
  gridToPixelH,
  gridToPixelY,
  pixelToGridH,
  pixelToGridW,
  pixelToGridX,
  pixelToGridY,
} from "./grid-metrics";

export const MIN_WIDGET_ROW_SPAN = 2;

export interface GridRectAnchor {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface GridRect {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface GridLayoutItem {
  id: string;
  pos_x: number;
  pos_y: number;
  width: number;
  height: number;
  widget_type: string;
}

export function snapWidgetRowSpan(height: number, decorative = false): number {
  if (decorative && height <= 1) return 1;
  if (height < MIN_WIDGET_ROW_SPAN) return MIN_WIDGET_ROW_SPAN;
  if (height % 2 !== 0) return height + 1;
  return height;
}

export function snapGridY(y: number, singleRowUnits = false): number {
  if (singleRowUnits) return Math.max(0, y);
  return Math.floor(Math.max(0, y) / MIN_WIDGET_ROW_SPAN) * MIN_WIDGET_ROW_SPAN;
}

export function usesSingleRowGridY(item: GridLayoutItem): boolean {
  return item.widget_type === "decorative_block" && item.height <= 1;
}

function collides(a: GridRect, b: GridRect): boolean {
  if (a.id === b.id) return false;
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;
}

function fromItem(item: GridLayoutItem): GridRect {
  return { id: item.id, x: item.pos_x, y: item.pos_y, w: item.width, h: item.height };
}

function resolveCollisions(rects: GridRect[], moved: GridRect): void {
  for (const other of rects) {
    if (other.id === moved.id) continue;
    if (collides(moved, other)) {
      other.y = moved.y + moved.h;
      resolveCollisions(rects, other);
    }
  }
}

function resolveAllOverlaps(rects: GridRect[]): void {
  rects.sort((a, b) => (a.y !== b.y ? a.y - b.y : a.x - b.x));
  const placed: GridRect[] = [];
  for (const rect of rects) {
    let safety = 0;
    while (safety++ < 100) {
      const blocker = placed.find((other) => collides(rect, other));
      if (!blocker) break;
      rect.y = blocker.y + blocker.h;
    }
    placed.push(rect);
  }
}

function applyRects<T extends GridLayoutItem>(originals: T[], rects: GridRect[]): T[] {
  return originals.map((orig) => {
    const rect = rects.find((r) => r.id === orig.id);
    if (!rect) return orig;
    if (
      rect.x === orig.pos_x &&
      rect.y === orig.pos_y &&
      rect.w === orig.width &&
      rect.h === orig.height
    ) {
      return orig;
    }
    return { ...orig, pos_x: rect.x, pos_y: rect.y, width: rect.w, height: rect.h };
  });
}

export class GridLayoutEngine {
  constructor(private readonly metrics: GridMetrics) {}

  contentHeight(layout: GridLayoutItem[]): number {
    if (layout.length === 0) return 0;
    let maxBottom = 0;
    for (const w of layout) {
      const single = usesSingleRowGridY(w);
      const bottom =
        gridToPixelY(w.pos_y, this.metrics, { singleRowUnits: single }) +
        gridToPixelH(w.height, this.metrics, { singleRowUnits: single });
      if (bottom > maxBottom) maxBottom = bottom;
    }
    return maxBottom;
  }

  fitLayoutToGrid(
    layout: GridLayoutItem[],
    options?: {
      isDecorativeBlock?: (item: GridLayoutItem) => boolean;
      minWidthForInstance?: (item: GridLayoutItem) => number;
    },
  ): GridLayoutItem[] {
    const rects = layout.map(fromItem);
    for (const rect of rects) {
      const item = layout.find((w) => w.id === rect.id)!;
      const decorative = options?.isDecorativeBlock?.(item) ?? false;
      const minW = options?.minWidthForInstance?.(item) ?? 1;
      if (!decorative && rect.w < minW) rect.w = minW;
      rect.w = Math.max(1, Math.min(this.metrics.columns, rect.w));
      rect.x = Math.max(0, Math.min(this.metrics.columns - rect.w, rect.x));
      rect.w = Math.max(1, Math.min(this.metrics.columns - rect.x, rect.w));
      rect.h = snapWidgetRowSpan(rect.h, decorative);
      const singleRow = decorative && rect.h <= 1;
      rect.y = snapGridY(rect.y, singleRow);
    }
    return applyRects(layout, rects);
  }

  normalizeLayout(
    layout: GridLayoutItem[],
    options?: {
      isDecorativeBlock?: (item: GridLayoutItem) => boolean;
      minWidthForInstance?: (item: GridLayoutItem) => number;
    },
  ): GridLayoutItem[] {
    if (layout.length === 0) return [];
    const fitted = this.fitLayoutToGrid(layout, options);
    const rects = fitted.map(fromItem);
    resolveAllOverlaps(rects);
    return applyRects(fitted, rects);
  }

  previewMoveWidget(
    layout: GridLayoutItem[],
    id: string,
    newX: number,
    newY: number,
    anchors: Record<string, GridRectAnchor>,
  ): GridLayoutItem[] {
    const rects = layout.map(fromItem);
    const moved = rects.find((r) => r.id === id);
    if (!moved) return layout;
    const item = layout.find((w) => w.id === id)!;
    const singleRow = usesSingleRowGridY(item);

    for (const rect of rects) {
      const anchor = anchors[rect.id];
      if (!anchor) continue;
      rect.x = anchor.x;
      rect.y = anchor.y;
      rect.w = anchor.w;
      rect.h = anchor.h;
    }

    moved.x = Math.max(0, Math.min(this.metrics.columns - moved.w, newX));
    moved.y = snapGridY(Math.max(0, newY), singleRow);
    resolveCollisions(rects, moved);
    return applyRects(layout, rects);
  }

  moveWidget(layout: GridLayoutItem[], id: string, newX: number, newY: number): GridLayoutItem[] {
    const rects = layout.map(fromItem);
    const moved = rects.find((r) => r.id === id);
    if (!moved) return layout;
    const item = layout.find((w) => w.id === id)!;
    const singleRow = usesSingleRowGridY(item);
    moved.x = Math.max(0, Math.min(this.metrics.columns - moved.w, newX));
    moved.y = snapGridY(Math.max(0, newY), singleRow);
    resolveCollisions(rects, moved);
    return applyRects(layout, rects);
  }

  resizeWidget(
    layout: GridLayoutItem[],
    id: string,
    newW: number,
    newH: number,
    options?: { minW?: number; minH?: number; maxW?: number; maxH?: number },
  ): GridLayoutItem[] {
    const rects = layout.map(fromItem);
    const target = rects.find((r) => r.id === id);
    if (!target) return layout;
    const decorative = target.w === 1 && target.h <= 1;
    const effMinW = options?.minW ?? 1;
    const effMinH = snapWidgetRowSpan(options?.minH ?? MIN_WIDGET_ROW_SPAN, decorative);
    const effMaxW = Math.max(
      effMinW,
      Math.min(options?.maxW ?? this.metrics.columns, this.metrics.columns - target.x),
    );
    const effMaxH = Math.max(effMinH, options?.maxH ?? 200);
    target.w = Math.max(effMinW, Math.min(effMaxW, newW));
    target.h = snapWidgetRowSpan(Math.max(effMinH, Math.min(effMaxH, newH)), decorative);
    resolveCollisions(rects, target);
    return applyRects(layout, rects);
  }

  pixelToGridFromDrag(
    origin: GridLayoutItem,
    offsetX: number,
    offsetY: number,
  ): { x: number; y: number } {
    const singleRow = usesSingleRowGridY(origin);
    const startX = origin.pos_x * (this.metrics.cellWidth + this.metrics.gap);
    const startY = gridToPixelY(origin.pos_y, this.metrics, { singleRowUnits: singleRow });
    return {
      x: pixelToGridX(startX + offsetX, this.metrics),
      y: pixelToGridY(startY + offsetY, this.metrics, { singleRowUnits: singleRow }),
    };
  }

  pixelToGridFromResize(
    origin: GridLayoutItem,
    offsetX: number,
    offsetY: number,
    options: { minW: number; minH: number; maxW: number },
  ): { w: number; h: number } {
    const singleRow = usesSingleRowGridY(origin);
    const startW = origin.width * this.metrics.cellWidth + Math.max(0, origin.width - 1) * this.metrics.gap;
    const startH = gridToPixelH(origin.height, this.metrics, { singleRowUnits: singleRow });
    return {
      w: pixelToGridW(startW + offsetX, this.metrics, options.minW, options.maxW),
      h: pixelToGridH(startH + offsetY, this.metrics, {
        minH: options.minH,
        decorative: origin.width === 1 && origin.height <= 1,
      }),
    };
  }

  static layoutHasOverlaps(layout: GridLayoutItem[]): boolean {
    const rects = layout.map(fromItem);
    for (let i = 0; i < rects.length; i++) {
      for (let j = i + 1; j < rects.length; j++) {
        if (collides(rects[i], rects[j])) return true;
      }
    }
    return false;
  }
}

export function defaultMetrics(containerWidth = 800): GridMetrics {
  return gridMetricsFromWidth(containerWidth);
}
