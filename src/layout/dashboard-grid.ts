import { invoke } from "@tauri-apps/api/core";
import { createGlassCard } from "../components/glass-card";
import {
  GridLayoutEngine,
  type GridLayoutItem,
  type GridRectAnchor,
  MIN_WIDGET_ROW_SPAN,
  usesSingleRowGridY,
} from "./grid-layout-engine";
import {
  gridContentHeight,
  gridContentWidth,
  gridMetricsFromViewport,
  gridToPixelH,
  gridToPixelW,
  gridToPixelX,
  gridToPixelY,
  placementCellHeight,
  placementRowStep,
} from "./grid-metrics";
import { parseWidgetColorSlot } from "../theme/color-utils";
import { getUiPrefs } from "../theme/theme-manager";
import type { WidgetCatalog, WidgetInstance, WidgetLayoutInput } from "../types";
import { catalogDisplayName, getCatalog, renderWidgetBody } from "../marketplace";

function parseWidgetConfig(configJson: string): { colors?: Record<string, string> } {
  try {
    return JSON.parse(configJson) as { colors?: Record<string, string> };
  } catch {
    return {};
  }
}

function widgetAccentColor(widget: WidgetInstance): string | undefined {
  const config = parseWidgetConfig(widget.config_json);
  const slot = config.colors?.primary ?? config.colors?.accent ?? config.colors?.border;
  return parseWidgetColorSlot(slot) ?? undefined;
}

function toLayoutItem(w: WidgetInstance): GridLayoutItem {
  return {
    id: w.id,
    pos_x: w.pos_x,
    pos_y: w.pos_y,
    width: w.width,
    height: w.height,
    widget_type: w.widget_type,
  };
}

function minWidthForType(widgetType: string, catalogMin?: number): number {
  if (widgetType === "decorative_block") return 1;
  return Math.max(2, catalogMin ?? 2);
}

function createResizeHandle(): HTMLElement {
  const handle = document.createElement("div");
  handle.className = "corner-resize-handle";
  handle.innerHTML = `
    <svg viewBox="0 0 44 44" aria-hidden="true">
      <line x1="15" y1="44" x2="44" y2="15" />
      <line x1="28" y1="44" x2="44" y2="28" />
    </svg>
  `;
  handle.setAttribute("role", "separator");
  handle.setAttribute("aria-label", "Resize widget");
  return handle;
}

export class DashboardGrid {
  private scrollHost: HTMLElement;
  private stack: HTMLElement;
  private overlay: HTMLElement;
  private onChanged: () => Promise<void>;

  private widgets: WidgetInstance[] = [];
  private catalog: WidgetCatalog | null = null;
  private metrics = gridMetricsFromViewport(800, 600);
  private engine = new GridLayoutEngine(this.metrics);

  private savedLayout: GridLayoutItem[] = [];
  private activeLayout: GridLayoutItem[] | null = null;
  private tileEls = new Map<string, HTMLElement>();

  private draggingId: string | null = null;
  private dragOrigin: GridLayoutItem | null = null;
  private dragOffset = { x: 0, y: 0 };
  private dragAnchors: Record<string, GridRectAnchor> = {};
  private dragSnapshot: GridLayoutItem[] = [];
  private dragPointerStart = { x: 0, y: 0 };

  private resizingId: string | null = null;
  private resizeOrigin: GridLayoutItem | null = null;
  private resizeOffset = { x: 0, y: 0 };
  private resizePointerStart = { x: 0, y: 0 };

  private globalPointerBound = false;
  private readonly onGlobalPointer = (event: PointerEvent) => this.handleGlobalPointer(event);

  constructor(host: HTMLElement, onChanged: () => Promise<void>) {
    this.onChanged = onChanged;
    host.className = "dashboard-grid-host";

    this.scrollHost = document.createElement("div");
    this.scrollHost.className = "dashboard-grid-scroll";

    this.stack = document.createElement("div");
    this.stack.className = "dashboard-grid-stack";

    this.overlay = document.createElement("div");
    this.overlay.className = "dashboard-grid-overlay hidden";

    this.stack.append(this.overlay);
    this.scrollHost.append(this.stack);
    host.replaceChildren(this.scrollHost);
  }

  get isLocked(): boolean {
    return getUiPrefs().layout_locked;
  }

  async load(): Promise<WidgetInstance[]> {
    this.catalog = await getCatalog();
    this.widgets = await invoke<WidgetInstance[]>("list_widgets");
    await this.maybePersistLayoutIfOverlapping();
    this.savedLayout = this.widgets.map(toLayoutItem);
    await this.render();
    return this.widgets;
  }

  private layoutOptions() {
    return {
      isDecorativeBlock: (item: GridLayoutItem) => item.widget_type === "decorative_block",
      minWidthForInstance: (item: GridLayoutItem) => {
        const def = this.catalog!.widgets.find((w) => w.widgetType === item.widget_type);
        return minWidthForType(item.widget_type, def?.defaultSize?.width);
      },
    };
  }

  private measureViewport(): { width: number; height: number } {
    const style = getComputedStyle(this.scrollHost);
    const padX =
      (parseFloat(style.paddingLeft) || 0) + (parseFloat(style.paddingRight) || 0);
    const padY =
      (parseFloat(style.paddingTop) || 0) + (parseFloat(style.paddingBottom) || 0);
    const width = this.scrollHost.clientWidth - padX;
    const height = this.scrollHost.clientHeight - padY;
    return {
      width: Math.max(320, width || 800),
      height: Math.max(200, height || 600),
    };
  }

  private syncMetrics(): void {
    const { width, height } = this.measureViewport();
    this.metrics = gridMetricsFromViewport(width, height);
    this.engine = new GridLayoutEngine(this.metrics);
  }

  getVisibleRowUnits(): number {
    this.syncMetrics();
    return this.metrics.visiblePlacementRows * MIN_WIDGET_ROW_SPAN;
  }

  private async maybePersistLayoutIfOverlapping(): Promise<void> {
    this.syncMetrics();
    const layout = this.widgets.map(toLayoutItem);
    if (!GridLayoutEngine.layoutHasOverlaps(layout)) return;

    const normalized = this.engine.normalizeLayout(layout, this.layoutOptions());
    const changed = normalized.some((n, i) => {
      const w = this.widgets[i];
      return n.pos_x !== w.pos_x || n.pos_y !== w.pos_y || n.width !== w.width || n.height !== w.height;
    });
    if (!changed) return;

    await invoke("update_widget_layouts", {
      layouts: normalized.map((n) => ({
        id: n.id,
        pos_x: n.pos_x,
        pos_y: n.pos_y,
        width: n.width,
        height: n.height,
      })),
    });
    this.widgets = await invoke<WidgetInstance[]>("list_widgets");
  }

  private currentLayout(): GridLayoutItem[] {
    return this.activeLayout ?? this.savedLayout;
  }

  private ensureOverlayInStack(): void {
    if (this.overlay.parentElement !== this.stack) {
      this.stack.prepend(this.overlay);
    }
  }

  private async render(): Promise<void> {
    const locked = this.isLocked;
    this.syncMetrics();
    const layout = this.currentLayout();
    this.ensureOverlayInStack();

    if (this.widgets.length === 0) {
      for (const el of [...this.stack.querySelectorAll(".widget-tile")]) {
        el.remove();
      }
      this.tileEls.clear();
      this.hideOverlay();
      this.stack.style.height = "240px";

      let empty = this.stack.querySelector<HTMLElement>(".widget-empty-state");
      if (!empty) {
        empty = document.createElement("div");
        empty.className = "widget-empty-state";
        empty.innerHTML = `
          <p class="widget-empty-state__title">No widgets yet</p>
          <p class="widget-empty-state__hint">Use <strong>Add widget</strong> in the toolbar to install tiles, then unlock the layout to move and resize them.</p>
        `;
        this.stack.appendChild(empty);
      }
      return;
    }

    this.stack.querySelector(".widget-empty-state")?.remove();

    const contentH = Math.max(
      this.engine.contentHeight(layout),
      gridContentHeight(this.metrics, this.metrics.visiblePlacementRows),
    );
    this.stack.style.width = `${gridContentWidth(this.metrics)}px`;
    this.stack.style.height = `${contentH}px`;

    if (this.isInteracting) {
      this.overlay.classList.remove("hidden");
      this.paintOverlay(layout);
    } else {
      this.hideOverlay();
    }

    const activeId = this.draggingId ?? this.resizingId;
    const sorted = [...layout].sort((a, b) => {
      if (a.id === activeId) return 1;
      if (b.id === activeId) return -1;
      return 0;
    });

    const seen = new Set<string>();
    for (const item of sorted) {
      seen.add(item.id);
      let tile = this.tileEls.get(item.id);
      if (!tile) {
        tile = await this.createTile(item);
        this.tileEls.set(item.id, tile);
        this.stack.appendChild(tile);
      }
      this.positionTile(tile, item);
      tile.classList.toggle("widget-tile--active", item.id === activeId);
      tile.classList.toggle("widget-tile--locked", locked);
      tile.querySelector<HTMLElement>(".corner-resize-handle")?.classList.toggle("hidden", locked);
    }

    for (const [id, el] of this.tileEls) {
      if (!seen.has(id)) {
        el.remove();
        this.tileEls.delete(id);
      }
    }
  }

  private hideOverlay(): void {
    this.overlay.classList.add("hidden");
    this.overlay.innerHTML = "";
  }

  private async createTile(item: GridLayoutItem): Promise<HTMLElement> {
    const widget = this.widgets.find((w) => w.id === item.id)!;
    const name = catalogDisplayName(this.catalog!, widget.widget_type);
    const body = document.createElement("div");
    body.className = "widget-body-inner";
    await renderWidgetBody(widget.widget_type, body);

    const shell = document.createElement("article");
    shell.className = "widget-tile";
    shell.dataset.id = item.id;
    if (widgetAccentColor(widget)) {
      shell.style.setProperty("--widget-accent", widgetAccentColor(widget)!);
    }

    const chrome = createGlassCard(body, {
      depth: "surface",
      className: "widget-tile__glass",
      padding: "0",
    });

    const header = document.createElement("header");
    header.className = "widget-tile__chrome";
    const isShellWidget = widget.widget_type.startsWith("desktop_");
    if (!isShellWidget) {
      header.innerHTML = `<span class="widget-tile__label">${name}</span>`;
    }
    if (!this.isLocked) {
      const removeBtn = document.createElement("button");
      removeBtn.type = "button";
      removeBtn.className = "widget-tile__remove";
      removeBtn.title = "Remove";
      removeBtn.setAttribute("aria-label", "Remove widget");
      removeBtn.innerHTML = `<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>`;
      removeBtn.onclick = async (event) => {
        event.stopPropagation();
        if (!confirm("Remove this widget?")) return;
        await invoke("remove_widget", { id: item.id });
        await this.onChanged();
      };
      header.appendChild(removeBtn);
    }

    const inner = chrome.querySelector<HTMLElement>(".glass-card__inner");
    if (inner) {
      chrome.insertBefore(header, inner);
      if (isShellWidget) {
        inner.classList.add("widget-body-inner--shell");
      }
    }

    const handle = createResizeHandle();
    handle.onpointerdown = (event) => {
      if (this.isLocked) return;
      event.stopPropagation();
      event.preventDefault();
      this.startResize(item.id, event);
    };

    shell.append(chrome, handle);

    header.onpointerdown = (event) => {
      if (this.isLocked) return;
      if ((event.target as HTMLElement).closest(".widget-tile__remove")) return;
      event.preventDefault();
      this.startDrag(item.id, event);
    };

    return shell;
  }

  private positionTile(tile: HTMLElement, item: GridLayoutItem): void {
    const widget = this.widgets.find((w) => w.id === item.id)!;
    const origin = this.draggingId === item.id && this.dragOrigin ? this.dragOrigin : widget;
    const singleRow = usesSingleRowGridY(origin);

    let left: number;
    let top: number;
    let width: number;
    let height: number;

    if (this.draggingId === item.id && this.dragOrigin) {
      left = gridToPixelX(this.dragOrigin.pos_x, this.metrics) + this.dragOffset.x;
      top =
        gridToPixelY(this.dragOrigin.pos_y, this.metrics, { singleRowUnits: singleRow }) +
        this.dragOffset.y;
      width = gridToPixelW(this.dragOrigin.width, this.metrics);
      height = gridToPixelH(this.dragOrigin.height, this.metrics, { singleRowUnits: singleRow });
    } else if (this.resizingId === item.id && this.resizeOrigin) {
      left = gridToPixelX(item.pos_x, this.metrics);
      top = gridToPixelY(item.pos_y, this.metrics, { singleRowUnits: singleRow });
      width = gridToPixelW(this.resizeOrigin.width, this.metrics) + this.resizeOffset.x;
      height =
        gridToPixelH(this.resizeOrigin.height, this.metrics, { singleRowUnits: singleRow }) +
        this.resizeOffset.y;
    } else {
      left = gridToPixelX(item.pos_x, this.metrics);
      top = gridToPixelY(item.pos_y, this.metrics, { singleRowUnits: singleRow });
      width = gridToPixelW(item.width, this.metrics);
      height = gridToPixelH(item.height, this.metrics, { singleRowUnits: singleRow });
    }

    tile.style.left = `${left}px`;
    tile.style.top = `${top}px`;
    tile.style.width = `${width}px`;
    tile.style.height = `${height}px`;
  }

  private paintOverlay(layout: GridLayoutItem[]): void {
    const activeId = this.draggingId ?? this.resizingId;
    const active = activeId ? layout.find((w) => w.id === activeId) : null;
    const cols = this.metrics.columns;
    const cellW = this.metrics.cellWidth;
    const gap = this.metrics.gap;
    const colStep = cellW + gap;
    const cellH = placementCellHeight(this.metrics);
    const rowStep = placementRowStep(this.metrics);
    const stackH = parseFloat(this.stack.style.height) || this.engine.contentHeight(layout);
    const gridW = gridContentWidth(this.metrics);

    const scrollTop = this.scrollHost.scrollTop;
    const viewH = this.scrollHost.clientHeight;
    const firstRow = Math.max(0, Math.floor(scrollTop / rowStep) - 1);
    const lastRow = Math.min(
      Math.max(8, Math.ceil(stackH / rowStep) + 1),
      Math.ceil((scrollTop + viewH) / rowStep) + 2,
    );

    const activeRect = active
      ? {
          left: active.pos_x,
          top: active.pos_y,
          right: active.pos_x + active.width,
          bottom: active.pos_y + active.height,
        }
      : null;

    let svg = `<svg class="dashboard-grid-overlay__svg" viewBox="0 0 ${gridW} ${stackH}" width="${gridW}" height="${stackH}">`;

    for (let placementRow = firstRow; placementRow < lastRow; placementRow++) {
      const cellGridTop = placementRow * MIN_WIDGET_ROW_SPAN;
      const cellGridBottom = cellGridTop + MIN_WIDGET_ROW_SPAN;
      for (let col = 0; col < cols; col++) {
        const highlighted =
          activeRect !== null &&
          col >= activeRect.left &&
          col < activeRect.right &&
          cellGridTop < activeRect.bottom &&
          cellGridBottom > activeRect.top;

        const x = col * colStep;
        const y = placementRow * rowStep;
        svg += `<rect class="grid-cell${highlighted ? " grid-cell--highlight" : ""}" x="${x}" y="${y}" width="${cellW}" height="${cellH}" rx="4" />`;
      }
    }

    if (active && activeRect) {
      const placementTop = Math.floor(activeRect.top / MIN_WIDGET_ROW_SPAN);
      const placementSpan = Math.ceil((activeRect.bottom - activeRect.top) / MIN_WIDGET_ROW_SPAN);
      const outlineX = activeRect.left * colStep - 1;
      const outlineY = placementTop * rowStep - 1;
      const outlineW = (activeRect.right - activeRect.left) * colStep - gap + 2;
      const outlineH = placementSpan * rowStep - gap + 2;
      svg += `<rect class="grid-predict" x="${outlineX}" y="${outlineY}" width="${outlineW}" height="${outlineH}" rx="6" />`;
    }

    svg += `</svg>`;
    this.overlay.innerHTML = svg;
  }

  private get isInteracting(): boolean {
    return this.draggingId !== null || this.resizingId !== null;
  }

  private attachGlobalPointer(): void {
    if (this.globalPointerBound) return;
    window.addEventListener("pointermove", this.onGlobalPointer);
    window.addEventListener("pointerup", this.onGlobalPointer);
    window.addEventListener("pointercancel", this.onGlobalPointer);
    this.globalPointerBound = true;
  }

  private detachGlobalPointer(): void {
    if (!this.globalPointerBound) return;
    window.removeEventListener("pointermove", this.onGlobalPointer);
    window.removeEventListener("pointerup", this.onGlobalPointer);
    window.removeEventListener("pointercancel", this.onGlobalPointer);
    this.globalPointerBound = false;
  }

  private handleGlobalPointer(event: PointerEvent): void {
    if (!this.isInteracting) return;
    if (event.type === "pointermove") {
      if (this.draggingId) this.updateDrag(event.clientX, event.clientY);
      if (this.resizingId) this.updateResize(event.clientX, event.clientY);
      return;
    }
    if (this.resizingId) void this.endResize();
    else if (this.draggingId) void this.endDrag();
  }

  private startDrag(id: string, event: PointerEvent): void {
    if (this.isInteracting) this.cancelInteraction();
    const origin = this.savedLayout.find((w) => w.id === id);
    if (!origin) return;
    this.draggingId = id;
    this.dragOrigin = { ...origin };
    this.dragOffset = { x: 0, y: 0 };
    this.dragSnapshot = this.savedLayout.map((w) => ({ ...w }));
    this.dragAnchors = Object.fromEntries(
      this.dragSnapshot.map((w) => [w.id, { x: w.pos_x, y: w.pos_y, w: w.width, h: w.height }]),
    );
    this.activeLayout = this.dragSnapshot.map((w) => ({ ...w }));
    this.dragPointerStart = { x: event.clientX, y: event.clientY };
    this.attachGlobalPointer();
    void this.render();
  }

  private updateDrag(clientX: number, clientY: number): void {
    if (!this.dragOrigin || !this.draggingId) return;
    this.dragOffset = {
      x: clientX - this.dragPointerStart.x,
      y: clientY - this.dragPointerStart.y,
    };
    const { x, y } = this.engine.pixelToGridFromDrag(
      this.dragOrigin,
      this.dragOffset.x,
      this.dragOffset.y,
    );
    this.activeLayout = this.engine.previewMoveWidget(
      this.dragSnapshot,
      this.draggingId,
      x,
      y,
      this.dragAnchors,
    );
    void this.render();
  }

  private async endDrag(): Promise<void> {
    if (!this.draggingId || !this.activeLayout) {
      this.cancelInteraction();
      return;
    }
    const layout = this.engine.fitLayoutToGrid(this.activeLayout, this.layoutOptions());
    await this.persistLayout(layout);
    this.cancelInteraction();
    await this.onChanged();
  }

  private startResize(id: string, event: PointerEvent): void {
    if (this.isInteracting) this.cancelInteraction();
    const origin = this.savedLayout.find((w) => w.id === id);
    if (!origin) return;
    this.resizingId = id;
    this.resizeOrigin = { ...origin };
    this.resizeOffset = { x: 0, y: 0 };
    this.activeLayout = this.savedLayout.map((w) => ({ ...w }));
    this.resizePointerStart = { x: event.clientX, y: event.clientY };
    this.attachGlobalPointer();
    void this.render();
  }

  private updateResize(clientX: number, clientY: number): void {
    if (!this.resizeOrigin || !this.resizingId || !this.activeLayout) return;
    this.resizeOffset = {
      x: clientX - this.resizePointerStart.x,
      y: clientY - this.resizePointerStart.y,
    };
    const originWidget = this.widgets.find((w) => w.id === this.resizingId)!;
    const def = this.catalog!.widgets.find((w) => w.widgetType === originWidget.widget_type);
    const minW = minWidthForType(originWidget.widget_type, def?.defaultSize?.width);
    const { w, h } = this.engine.pixelToGridFromResize(
      this.resizeOrigin,
      this.resizeOffset.x,
      this.resizeOffset.y,
      {
        minW,
        minH: 2,
        maxW: this.metrics.columns - this.resizeOrigin.pos_x,
      },
    );
    this.activeLayout = this.engine.resizeWidget(this.activeLayout, this.resizingId, w, h, {
      minW,
      minH: 2,
      maxW: this.metrics.columns - this.resizeOrigin.pos_x,
    });
    void this.render();
  }

  private async endResize(): Promise<void> {
    if (!this.resizingId || !this.activeLayout) {
      this.cancelInteraction();
      return;
    }
    const layout = this.engine.fitLayoutToGrid(this.activeLayout, this.layoutOptions());
    await this.persistLayout(layout);
    this.cancelInteraction();
    await this.onChanged();
  }

  private cancelInteraction(): void {
    this.draggingId = null;
    this.dragOrigin = null;
    this.dragOffset = { x: 0, y: 0 };
    this.dragAnchors = {};
    this.dragSnapshot = [];
    this.resizingId = null;
    this.resizeOrigin = null;
    this.resizeOffset = { x: 0, y: 0 };
    this.activeLayout = null;
    this.detachGlobalPointer();
    this.hideOverlay();
    void this.render();
  }

  private async persistLayout(layout: GridLayoutItem[]): Promise<void> {
    const layouts: WidgetLayoutInput[] = layout.map((n) => ({
      id: n.id,
      pos_x: n.pos_x,
      pos_y: n.pos_y,
      width: n.width,
      height: n.height,
    }));
    await invoke("update_widget_layouts", { layouts });
    this.savedLayout = layout;
  }

  async refresh(): Promise<void> {
    await this.load();
  }

  scrollToTop(): void {
    this.scrollHost.scrollTop = 0;
  }

  onResize(): void {
    void this.render();
  }
}

export async function mountDashboardGrid(
  container: HTMLElement,
  onChanged: () => Promise<void>,
): Promise<DashboardGrid> {
  const grid = new DashboardGrid(container, onChanged);
  await grid.load();
  const scrollHost = container.querySelector<HTMLElement>(".dashboard-grid-scroll");
  if (scrollHost) {
    const ro = new ResizeObserver(() => grid.onResize());
    ro.observe(scrollHost);
  }
  return grid;
}
