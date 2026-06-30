import { DockviewComponent, type DockviewApi } from "dockview";
import "dockview/dist/styles/dockview.css";

import { mountAssistPanel } from "../assist/assist-panel";
import { renderDashboardGrid } from "../dashboard";
import { renderMarketplace } from "../marketplace";
import { renderSyncPanel } from "../sync-panel";
import { renderTimeline } from "../timeline";
import { loadShellLayout, saveShellLayout } from "../theme/theme-manager";

export type PanelId = "timeline" | "grid" | "marketplace" | "assist" | "sync";

type PanelMount = (el: HTMLElement) => void | (() => void) | Promise<void | (() => void)>;

const PANEL_DEFS: Record<PanelId, { title: string; mount: PanelMount }> = {
  timeline: {
    title: "Timeline",
    mount: (el) => {
      void renderTimeline(el);
    },
  },
  grid: {
    title: "Widgets",
    mount: (el) => {
      let refresh = async () => {
        await renderDashboardGrid(el, refresh);
      };
      void refresh();
    },
  },
  marketplace: {
    title: "Marketplace",
    mount: (el) => {
      let refresh = async () => {
        const { invoke } = await import("@tauri-apps/api/core");
        const widgets = await invoke<import("../types").WidgetInstance[]>("list_widgets");
        await renderMarketplace(el, widgets, refresh);
      };
      void refresh();
    },
  },
  assist: {
    title: "Brian",
    mount: (el) => mountAssistPanel(el),
  },
  sync: {
    title: "Sync",
    mount: (el) => renderSyncPanel(el),
  },
};

const disposers = new Map<string, () => void>();

async function mountPanel(panelId: PanelId, container: HTMLElement): Promise<void> {
  disposers.get(panelId)?.();
  disposers.delete(panelId);
  const result = PANEL_DEFS[panelId].mount(container);
  if (result instanceof Promise) {
    const dispose = await result;
    if (typeof dispose === "function") disposers.set(panelId, dispose);
  } else if (typeof result === "function") {
    disposers.set(panelId, result);
  }
}

function buildDefaultLayout(api: DockviewApi): void {
  api.addPanel({
    id: "timeline",
    component: "timeline",
    title: "Timeline",
    position: { direction: "left" },
    initialWidth: 320,
  });
  api.addPanel({
    id: "grid",
    component: "grid",
    title: "Widgets",
    position: { referencePanel: "timeline", direction: "right" },
  });
  api.addPanel({
    id: "marketplace",
    component: "marketplace",
    title: "Marketplace",
    position: { referencePanel: "grid", direction: "within" },
  });
  api.addPanel({
    id: "assist",
    component: "assist",
    title: "Brian",
    position: { referencePanel: "grid", direction: "right" },
    initialWidth: 360,
  });
  api.addPanel({
    id: "sync",
    component: "sync",
    title: "Sync",
    position: { referencePanel: "grid", direction: "below" },
    initialHeight: 160,
  });
}

export interface DockShell {
  api: DockviewApi;
  refreshAll: () => void;
  togglePanel: (id: PanelId) => void;
  dispose: () => void;
}

export async function createDockShell(host: HTMLElement): Promise<DockShell> {
  host.classList.add("dock-host", "dockview-theme-dark");

  const dockview = new DockviewComponent(host, {
    createComponent: (options) => {
      const panelId = options.name as PanelId;
      const element = document.createElement("div");
      element.className = "dock-panel-root";
      return {
        element,
        init: () => {
          void mountPanel(panelId, element);
        },
        dispose: () => {
          disposers.get(panelId)?.();
          disposers.delete(panelId);
        },
      };
    },
    className: "core-dockview",
  });

  const saved = await loadShellLayout();
  try {
    if (saved) {
      dockview.fromJSON(JSON.parse(saved));
    } else {
      buildDefaultLayout(dockview.api);
    }
  } catch {
    dockview.clear();
    buildDefaultLayout(dockview.api);
  }

  let saveTimer: number | undefined;
  const scheduleSave = () => {
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(() => {
      void saveShellLayout(JSON.stringify(dockview.toJSON()));
    }, 400);
  };

  dockview.onDidMutateLayout(scheduleSave);

  const api = dockview.api;

  return {
    api,
    refreshAll() {
      for (const id of Object.keys(PANEL_DEFS) as PanelId[]) {
        const panel = api.getPanel(id);
        const el = panel?.view?.content?.element as HTMLElement | undefined;
        if (el) void mountPanel(id, el);
      }
    },
    togglePanel(id) {
      const panel = api.getPanel(id);
      if (panel?.api.isActive) {
        panel.api.close();
        return;
      }
      if (!panel) {
        api.addPanel({ id, component: id, title: PANEL_DEFS[id].title });
      } else {
        panel.api.setActive();
      }
    },
    dispose() {
      for (const d of disposers.values()) d();
      disposers.clear();
      dockview.dispose();
    },
  };
}

export function wireShellShortcuts(shell: DockShell): void {
  document.addEventListener("keydown", (event) => {
    if (!event.ctrlKey && !event.metaKey) return;
    if (event.key.toLowerCase() === "b") {
      event.preventDefault();
      shell.togglePanel("timeline");
    }
    if (event.key.toLowerCase() === "j") {
      event.preventDefault();
      shell.togglePanel("sync");
    }
  });
}
