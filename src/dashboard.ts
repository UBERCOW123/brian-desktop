import { invoke } from "@tauri-apps/api/core";
import { CELL_PX, COLUMN_COUNT, GRID_GAP, type WidgetInstance, type WidgetLayoutInput } from "./types";
import { catalogDisplayName, getCatalog, renderWidgetBody } from "./marketplace";

type DragState = {
  id: string;
  startX: number;
  startY: number;
  originPosX: number;
  originPosY: number;
};

let dragState: DragState | null = null;

export async function renderDashboardGrid(
  container: HTMLElement,
  onLayoutSaved: () => Promise<void>,
): Promise<WidgetInstance[]> {
  const widgets = await invoke<WidgetInstance[]>("list_widgets");
  const catalog = await getCatalog();

  if (widgets.length === 0) {
    container.innerHTML = `<p class="empty-state">No widgets installed. Open Marketplace to add tiles.</p>`;
    return widgets;
  }

  const maxRow = widgets.reduce((m, w) => Math.max(m, w.pos_y + w.height), 0);
  const rowCount = Math.max(maxRow, 8);

  container.innerHTML = `
    <div class="dashboard-grid" style="--rows: ${rowCount}">
      ${widgets
        .map((w) => {
          const name = catalogDisplayName(catalog, w.widget_type);
          return `
            <article class="widget-tile"
              data-id="${w.id}"
              style="grid-column: ${w.pos_x + 1} / span ${w.width}; grid-row: ${w.pos_y + 1} / span ${w.height};"
              draggable="false">
              <header class="widget-header" data-drag-handle>
                <span>${name}</span>
                <button type="button" class="icon-btn" data-remove="${w.id}" title="Remove">×</button>
              </header>
              <div class="widget-body" data-body="${w.widget_type}"></div>
            </article>
          `;
        })
        .join("")}
    </div>
  `;

  const grid = container.querySelector<HTMLElement>(".dashboard-grid")!;

  for (const tile of grid.querySelectorAll<HTMLElement>(".widget-tile")) {
    const widgetType = tile.querySelector<HTMLElement>("[data-body]")!.dataset.body!;
    await renderWidgetBody(widgetType, tile.querySelector<HTMLElement>(".widget-body")!);
  }

  grid.querySelectorAll<HTMLButtonElement>("[data-remove]").forEach((btn) => {
    btn.onclick = async (event) => {
      event.stopPropagation();
      if (!confirm("Remove this widget?")) return;
      await invoke("remove_widget", { id: btn.dataset.remove });
      await onLayoutSaved();
    };
  });

  grid.querySelectorAll<HTMLElement>("[data-drag-handle]").forEach((handle) => {
    handle.addEventListener("pointerdown", (event) => {
      if ((event.target as HTMLElement).closest("[data-remove]")) return;
      const tile = handle.closest<HTMLElement>(".widget-tile")!;
      const widget = widgets.find((w) => w.id === tile.dataset.id)!;
      dragState = {
        id: widget.id,
        startX: event.clientX,
        startY: event.clientY,
        originPosX: widget.pos_x,
        originPosY: widget.pos_y,
      };
      handle.setPointerCapture(event.pointerId);
      tile.classList.add("dragging");
    });

    handle.addEventListener("pointermove", (event) => {
      if (!dragState || dragState.id !== handle.closest<HTMLElement>(".widget-tile")!.dataset.id) return;
      const tile = handle.closest<HTMLElement>(".widget-tile")!;
      const widget = widgets.find((w) => w.id === dragState!.id)!;
      const rect = grid.getBoundingClientRect();
      const colWidth = (rect.width - GRID_GAP * (COLUMN_COUNT - 1)) / COLUMN_COUNT;
      const rowHeight = CELL_PX;
      const dx = event.clientX - dragState.startX;
      const dy = event.clientY - dragState.startY;
      const newPosX = Math.max(
        0,
        Math.min(COLUMN_COUNT - widget.width, dragState.originPosX + Math.round(dx / (colWidth + GRID_GAP))),
      );
      const newPosY = Math.max(0, dragState.originPosY + Math.round(dy / (rowHeight + GRID_GAP)));
      tile.style.gridColumn = `${newPosX + 1} / span ${widget.width}`;
      tile.style.gridRow = `${newPosY + 1} / span ${widget.height}`;
    });

    handle.addEventListener("pointerup", async (event) => {
      const tile = handle.closest<HTMLElement>(".widget-tile")!;
      tile.classList.remove("dragging");
      if (!dragState || dragState.id !== tile.dataset.id) return;

      const widget = widgets.find((w) => w.id === dragState!.id)!;
      const style = tile.style;
      const colMatch = /grid-column:\s*(\d+)\s*\/\s*span\s*(\d+)/.exec(style.gridColumn);
      const rowMatch = /grid-row:\s*(\d+)\s*\/\s*span\s*(\d+)/.exec(style.gridRow);
      const pos_x = colMatch ? Number(colMatch[1]) - 1 : widget.pos_x;
      const width = colMatch ? Number(colMatch[2]) : widget.width;
      const pos_y = rowMatch ? Number(rowMatch[1]) - 1 : widget.pos_y;
      const height = rowMatch ? Number(rowMatch[2]) : widget.height;

      dragState = null;
      handle.releasePointerCapture(event.pointerId);

      if (pos_x === widget.pos_x && pos_y === widget.pos_y) return;

      const layout: WidgetLayoutInput = {
        id: widget.id,
        pos_x,
        pos_y,
        width,
        height,
      };
      await invoke("update_widget_layouts", { layouts: [layout] });
      await onLayoutSaved();
    });
  });

  return widgets;
}
