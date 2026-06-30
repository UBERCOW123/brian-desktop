import { describe, expect, it } from "vitest";
import { GRID_GAP } from "../types";
import {
  CELL_WIDTH,
  MIN_GRID_COLUMNS,
  PLACEMENT_CELL_HEIGHT,
  columnsFromWidth,
  gridContentWidth,
  gridMetricsFromViewport,
  gridToPixelW,
  placementRowStep,
  visibleRowUnitsForViewport,
} from "./grid-metrics";
import { MIN_WIDGET_ROW_SPAN } from "./grid-layout-engine";

describe("gridMetricsFromViewport", () => {
  it("uses fixed 80×120 cells and 8px gap", () => {
    const metrics = gridMetricsFromViewport(1280, 720);
    expect(metrics.cellWidth).toBe(CELL_WIDTH);
    expect(metrics.placementCellHeight).toBe(PLACEMENT_CELL_HEIGHT);
    expect(metrics.gap).toBe(GRID_GAP);
    expect(placementRowStep(metrics)).toBe(128);
  });

  it("grows column count on wide viewports (cells stay 80px)", () => {
    const metrics = gridMetricsFromViewport(1280, 720);
    expect(metrics.columns).toBeGreaterThan(MIN_GRID_COLUMNS);
    expect(metrics.columns).toBe(columnsFromWidth(1280));
    expect(gridContentWidth(metrics)).toBe(gridToPixelW(metrics.columns, metrics));
    expect(gridContentWidth(metrics)).toBeLessThanOrEqual(1280);
  });

  it("keeps minimum 8 columns on narrow viewports", () => {
    const metrics = gridMetricsFromViewport(360, 640);
    expect(metrics.columns).toBe(MIN_GRID_COLUMNS);
    expect(gridContentWidth(metrics)).toBe(696);
  });

  it("derives 5+ visible placement rows at 720px height", () => {
    const metrics = gridMetricsFromViewport(1280, 720);
    expect(metrics.visiblePlacementRows).toBeGreaterThanOrEqual(5);
  });

  it("ultrawide adds more columns; cell size unchanged", () => {
    const metrics = gridMetricsFromViewport(2400, 1200);
    expect(metrics.cellWidth).toBe(80);
    expect(metrics.gap).toBe(GRID_GAP);
    expect(metrics.columns).toBeGreaterThan(20);
    expect(gridContentWidth(metrics)).toBeGreaterThan(2000);
  });

  it("minimum 4 visible placement rows on short viewports", () => {
    const metrics = gridMetricsFromViewport(360, 300);
    expect(metrics.visiblePlacementRows).toBeGreaterThanOrEqual(4);
  });
});

describe("visibleRowUnitsForViewport", () => {
  it("returns visiblePlacementRows × 2 row units", () => {
    const metrics = gridMetricsFromViewport(1280, 720);
    expect(visibleRowUnitsForViewport(720, metrics)).toBe(
      metrics.visiblePlacementRows * MIN_WIDGET_ROW_SPAN,
    );
  });
});
