import { invoke } from "@tauri-apps/api/core";
import { createGlassCard } from "./components/glass-card";
import { AssistWidget } from "./react/AssistWidget";
import { mountReact } from "./react/mount";
import { renderSyncPanel } from "./sync-panel";
import { renderTimeline } from "./timeline";
import type { WidgetCatalog, WidgetCatalogEntry, WidgetInstance } from "./types";
import { escapeHtml } from "./types";

let catalogCache: WidgetCatalog | null = null;

export async function getCatalog(): Promise<WidgetCatalog> {
  if (!catalogCache) {
    catalogCache = await invoke<WidgetCatalog>("widget_catalog");
  }
  return catalogCache;
}

export function catalogDisplayName(catalog: WidgetCatalog, widgetType: string): string {
  return catalog.widgets.find((w) => w.widgetType === widgetType)?.displayName ?? widgetType;
}

export async function renderMarketplace(
  container: HTMLElement,
  installed: WidgetInstance[],
  onInstalled: () => Promise<void>,
): Promise<void> {
  container.className = "marketplace-panel";
  const catalog = await getCatalog();
  const categories = [...new Set(catalog.widgets.map((w) => w.category).filter(Boolean))] as string[];

  const filters = document.createElement("div");
  filters.className = "marketplace-filters";
  filters.innerHTML = `<button type="button" class="chip chip--active" data-category="">All</button>`;
  for (const c of categories) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "chip";
    btn.dataset.category = c;
    btn.textContent = c;
    filters.appendChild(btn);
  }

  const grid = document.createElement("div");
  grid.className = "marketplace-grid";

  container.replaceChildren(filters, grid);

  const renderList = (category: string) => {
    const filtered = category
      ? catalog.widgets.filter((w) => w.category === category)
      : catalog.widgets;
    grid.replaceChildren();
    for (const entry of filtered) {
      grid.appendChild(buildMarketCard(entry, installed, onInstalled));
    }
  };

  filters.querySelectorAll<HTMLButtonElement>(".chip").forEach((chip) => {
    chip.onclick = () => {
      filters.querySelectorAll(".chip").forEach((c) => c.classList.remove("chip--active"));
      chip.classList.add("chip--active");
      renderList(chip.dataset.category ?? "");
    };
  });

  renderList("");
}

function buildMarketCard(
  entry: WidgetCatalogEntry,
  installed: WidgetInstance[],
  onInstalled: () => Promise<void>,
): HTMLElement {
  const installedCount = installed.filter((w) => w.widget_type === entry.widgetType).length;
  const canInstall = entry.allowMultiple || installedCount === 0;
  const size = entry.defaultSize ?? { width: 2, height: 2 };

  const inner = document.createElement("div");
  inner.innerHTML = `
    <h4 class="market-card__title">${escapeHtml(entry.displayName)}</h4>
    <p class="market-card__meta">${escapeHtml(entry.widgetType)} · ${size.width}×${size.height}</p>
    ${entry.category ? `<span class="badge">${escapeHtml(entry.category)}</span>` : ""}
    <button type="button" class="btn btn--small" data-install ${canInstall ? "" : "disabled"}>
      ${canInstall ? "Install" : "Installed"}
    </button>
  `;

  const card = createGlassCard(inner, { depth: "surface", className: "market-card", padding: "0" });

  const btn = card.querySelector<HTMLButtonElement>("[data-install]");
  if (btn && canInstall) {
    btn.onclick = async () => {
      btn.disabled = true;
      try {
        await invoke("install_widget", { widgetType: entry.widgetType });
        catalogCache = null;
        await onInstalled();
      } catch (err) {
        alert(String(err));
      } finally {
        btn.disabled = false;
      }
    };
  }
  return card;
}

export async function renderWidgetBody(widgetType: string, body: HTMLElement): Promise<void> {
  body.className = "widget-body-inner widget-body-inner--fill";

  if (widgetType === "desktop_assist") {
    mountReact(body, <AssistWidget />);
    return;
  }

  if (widgetType === "desktop_timeline") {
    await renderTimeline(body);
    return;
  }

  if (widgetType === "desktop_sync") {
    renderSyncPanel(body);
    return;
  }

  if (widgetType === "task_queue") {
    const tasks = await invoke<import("./types").TaskQueueItem[]>("list_tasks", {
      includeCompleted: false,
    });
    if (tasks.length === 0) {
      body.innerHTML = `<p class="widget-empty">No active tasks</p>`;
      return;
    }
    body.innerHTML = `<ul class="widget-task-list">${tasks
      .slice(0, 6)
      .map(
        (t) =>
          `<li>${escapeHtml(t.display_title || t.title)}${t.is_overdue ? " <span class='overdue'>overdue</span>" : ""}</li>`,
      )
      .join("")}</ul>`;
    return;
  }

  if (widgetType === "clock_widget") {
    const tick = () => {
      body.textContent = new Date().toLocaleTimeString();
    };
    tick();
    window.setInterval(tick, 1000);
    return;
  }

  body.innerHTML = `<p class="widget-empty">${escapeHtml(widgetType)}</p>`;
}
