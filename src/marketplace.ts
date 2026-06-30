import { invoke } from "@tauri-apps/api/core";
import type { TaskQueueItem, WidgetCatalog, WidgetCatalogEntry, WidgetInstance } from "./types";
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
  const catalog = await getCatalog();
  const categories = [...new Set(catalog.widgets.map((w) => w.category).filter(Boolean))] as string[];

  container.innerHTML = `
    <div class="marketplace-filters">
      <button type="button" class="chip active" data-category="">All</button>
      ${categories
        .map((c) => `<button type="button" class="chip" data-category="${escapeHtml(c)}">${escapeHtml(c)}</button>`)
        .join("")}
    </div>
    <div class="marketplace-grid" id="marketplace-grid"></div>
  `;

  const grid = container.querySelector<HTMLElement>("#marketplace-grid")!;
  const chips = container.querySelectorAll<HTMLButtonElement>(".chip");

  const renderList = (category: string) => {
    const filtered = category
      ? catalog.widgets.filter((w) => w.category === category)
      : catalog.widgets;
    grid.innerHTML = filtered.map((entry) => cardHtml(entry, installed)).join("");
    grid.querySelectorAll<HTMLButtonElement>("[data-install]").forEach((btn) => {
      btn.onclick = async () => {
        const type = btn.dataset.install!;
        btn.disabled = true;
        try {
          await invoke("install_widget", { widgetType: type });
          await onInstalled();
        } catch (err) {
          alert(String(err));
        } finally {
          btn.disabled = false;
        }
      };
    });
  };

  chips.forEach((chip) => {
    chip.onclick = () => {
      chips.forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      renderList(chip.dataset.category ?? "");
    };
  });

  renderList("");
}

function cardHtml(entry: WidgetCatalogEntry, installed: WidgetInstance[]): string {
  const installedCount = installed.filter((w) => w.widget_type === entry.widgetType).length;
  const canInstall = entry.allowMultiple || installedCount === 0;
  const size = entry.defaultSize ?? { width: 2, height: 2 };
  return `
    <article class="market-card">
      <h4>${escapeHtml(entry.displayName)}</h4>
      <p class="meta">${escapeHtml(entry.widgetType)} · ${size.width}×${size.height}</p>
      ${entry.category ? `<span class="badge">${escapeHtml(entry.category)}</span>` : ""}
      <button type="button" class="btn btn-small" data-install="${escapeHtml(entry.widgetType)}"
        ${canInstall ? "" : "disabled title='Already installed'"}>
        ${canInstall ? "Install" : "Installed"}
      </button>
    </article>
  `;
}

export async function renderWidgetBody(
  widgetType: string,
  body: HTMLElement,
): Promise<void> {
  if (widgetType === "task_queue") {
    const tasks = await invoke<TaskQueueItem[]>("list_tasks", { includeCompleted: false });
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
