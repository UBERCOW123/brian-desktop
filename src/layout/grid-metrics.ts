import { COLUMN_COUNT, GRID_GAP } from "../types";
import { MIN_WIDGET_ROW_SPAN } from "./grid-layout-engine";

export const CELL_WIDTH = 80;
export const PLACEMENT_CELL_HEIGHT = 120;
/** CORE template uses 8 columns; grid grows beyond this on wide viewports. */
export const MIN_GRID_COLUMNS = COLUMN_COUNT;
/** Minimum visible placement rows on the workbench. */
export const MIN_VISIBLE_PLACEMENT_ROWS = 4;

export interface GridMetrics {
  columns: number;
  cellWidth: number;
  placementCellHeight: number;
  gap: number;
  containerWidth: number;
  visiblePlacementRows: number;
}

/** How many 80px columns (+ 8px gaps) fit in [containerWidth]. */
export function columnsFromWidth(containerWidth: number): number {
  const width = Math.max(320, containerWidth);
  const colStep = CELL_WIDTH + GRID_GAP;
  return Math.max(MIN_GRID_COLUMNS, Math.floor((width + GRID_GAP) / colStep));
}

function visiblePlacementRowsFromHeight(containerHeight: number): number {
  const height = Math.max(200, containerHeight);
  const rowStep = PLACEMENT_CELL_HEIGHT + GRID_GAP;
  return Math.max(MIN_VISIBLE_PLACEMENT_ROWS, Math.floor((height + GRID_GAP) / rowStep));
}

export function gridMetricsFromViewport(
  containerWidth: number,
  containerHeight: number,
): GridMetrics {
  const width = Math.max(320, containerWidth);
  return {
    columns: columnsFromWidth(width),
    cellWidth: CELL_WIDTH,
    placementCellHeight: PLACEMENT_CELL_HEIGHT,
    gap: GRID_GAP,
    containerWidth: width,
    visiblePlacementRows: visiblePlacementRowsFromHeight(containerHeight),
  };
}

/** Width-only fallback (uses default visible row count). */
export function gridMetricsFromWidth(containerWidth: number): GridMetrics {
  return gridMetricsFromViewport(containerWidth, 720);
}

export function visibleRowUnitsForViewport(_containerHeight: number, metrics: GridMetrics): number {
  return metrics.visiblePlacementRows * MIN_WIDGET_ROW_SPAN;
}

export function placementCellHeight(metrics: GridMetrics): number {
  return metrics.placementCellHeight;
}

export function placementRowStep(metrics: GridMetrics): number {
  return metrics.placementCellHeight + metrics.gap;
}

export function columnStep(metrics: GridMetrics): number {
  return metrics.cellWidth + metrics.gap;
}

export function gridToPixelX(gx: number, metrics: GridMetrics): number {
  return gx * columnStep(metrics);
}

export function gridToPixelW(gw: number, metrics: GridMetrics): number {
  return gw * metrics.cellWidth + Math.max(0, gw - 1) * metrics.gap;
}

/** Total pixel width spanned by all visible columns. */
export function gridContentWidth(metrics: GridMetrics): number {
  return gridToPixelW(metrics.columns, metrics);
}

export function gridContentHeight(metrics: GridMetrics, placementRows: number): number {
  if (placementRows <= 0) return 0;
  return placementRows * metrics.placementCellHeight + (placementRows - 1) * metrics.gap;
}

export function gridToPixelY(
  gy: number,
  metrics: GridMetrics,
  options?: { singleRowUnits?: boolean },
): number {
  if (options?.singleRowUnits) {
    const rowUnit = (metrics.placementCellHeight - metrics.gap) / MIN_WIDGET_ROW_SPAN;
    return gy * (rowUnit + metrics.gap);
  }
  const placementRow = Math.floor(gy / MIN_WIDGET_ROW_SPAN);
  return placementRow * placementRowStep(metrics);
}

export function gridToPixelH(
  gh: number,
  metrics: GridMetrics,
  options?: { singleRowUnits?: boolean },
): number {
  if (options?.singleRowUnits) {
    const rowUnit = (metrics.placementCellHeight - metrics.gap) / MIN_WIDGET_ROW_SPAN;
    return gh * rowUnit + Math.max(0, gh - 1) * metrics.gap;
  }
  const placementRows = Math.ceil(gh / MIN_WIDGET_ROW_SPAN);
  if (placementRows <= 0) return 0;
  return gridContentHeight(metrics, placementRows);
}

export function pixelToGridX(px: number, metrics: GridMetrics): number {
  return Math.max(
    0,
    Math.min(metrics.columns - 1, Math.round(px / columnStep(metrics))),
  );
}

export function pixelToGridY(
  py: number,
  metrics: GridMetrics,
  options?: { singleRowUnits?: boolean },
): number {
  if (options?.singleRowUnits) {
    const rowUnit = (metrics.placementCellHeight - metrics.gap) / MIN_WIDGET_ROW_SPAN;
    return Math.max(0, Math.round(py / (rowUnit + metrics.gap)));
  }
  const step = placementRowStep(metrics);
  const placementRow = Math.floor(Math.max(0, py) / step);
  return placementRow * MIN_WIDGET_ROW_SPAN;
}

export function pixelToGridW(
  pw: number,
  metrics: GridMetrics,
  minW = 1,
  maxW?: number,
): number {
  const cap = maxW ?? metrics.columns;
  return Math.max(
    minW,
    Math.min(cap, Math.round((pw + metrics.gap) / columnStep(metrics))),
  );
}

export function pixelToGridH(
  ph: number,
  metrics: GridMetrics,
  options?: { minH?: number; maxH?: number; decorative?: boolean },
): number {
  const step = placementRowStep(metrics);
  const raw = Math.round((ph + metrics.gap) / step) * MIN_WIDGET_ROW_SPAN;
  const minH = options?.minH ?? MIN_WIDGET_ROW_SPAN;
  const maxH = options?.maxH ?? 200;
  let snapped = raw;
  if (!options?.decorative) {
    if (snapped < minH) snapped = minH;
    if (snapped % 2 !== 0) snapped += 1;
  }
  return Math.max(minH, Math.min(maxH, snapped));
}
