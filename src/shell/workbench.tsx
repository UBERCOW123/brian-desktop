import { invoke } from "@tauri-apps/api/core";
import type { DashboardGrid } from "../layout/dashboard-grid";
import { mountDashboardGrid } from "../layout/dashboard-grid";
import { getUiPrefs, saveThemePref } from "../theme/theme-manager";
import { mountReact, rerenderReact } from "../react/mount";
import { AddWidgetDrawer } from "../react/AddWidgetDrawer";
import { GridToolbar } from "../react/GridToolbar";

export type Workbench = {
  refresh: () => Promise<void>;
  setLockUi: (locked: boolean) => void;
};

export async function createWorkbench(root: HTMLElement): Promise<Workbench> {
  await invoke("seed_desktop_layout_if_empty");

  root.className = "workbench";

  const toolbarHost = document.createElement("div");
  toolbarHost.className = "workbench__toolbar-host";

  const gridHost = document.createElement("div");
  gridHost.className = "workbench__grid-host";

  const drawerHost = document.createElement("div");
  drawerHost.className = "workbench__drawer-host";

  root.replaceChildren(toolbarHost, gridHost, drawerHost);

  let grid: DashboardGrid | null = null;
  let layoutLocked = getUiPrefs().layout_locked;
  let drawerOpen = false;

  const refreshGrid = async (): Promise<DashboardGrid> => {
    if (!grid) {
      grid = await mountDashboardGrid(gridHost, refreshAll);
    } else {
      await grid.refresh();
    }
    return grid;
  };

  async function refreshAll(): Promise<void> {
    await refreshGrid();
  }

  const renderDrawer = () => {
    rerenderReact(
      drawerHost,
      <AddWidgetDrawer
        open={drawerOpen}
        onClose={() => {
          drawerOpen = false;
          renderToolbar();
          renderDrawer();
        }}
        onInstalled={async () => {
          await refreshAll();
        }}
      />,
    );
  };

  const renderToolbar = () => {
    rerenderReact(
      toolbarHost,
      <GridToolbar
        layoutLocked={layoutLocked}
        drawerOpen={drawerOpen}
        onLockChange={async (locked) => {
          layoutLocked = locked;
          await saveThemePref({ layout_locked: locked });
          renderToolbar();
          await refreshGrid();
        }}
        onAddWidget={() => {
          drawerOpen = !drawerOpen;
          renderToolbar();
          renderDrawer();
        }}
        onResetLayout={async () => {
          if (!confirm("Reset the workbench to the default desktop layout?")) return;
          await invoke("reset_desktop_layout");
          await refreshAll();
        }}
      />,
    );
  };

  mountReact(toolbarHost, null);
  mountReact(drawerHost, null);

  const setLockUi = (locked: boolean) => {
    layoutLocked = locked;
    renderToolbar();
  };

  renderToolbar();
  renderDrawer();
  const mountedGrid = await refreshGrid();
  const visibleRowUnits = mountedGrid.getVisibleRowUnits();
  const repaired = await invoke<boolean>("repair_workbench_layout", { visibleRowUnits });
  if (repaired) {
    await mountedGrid.refresh();
  }
  mountedGrid.scrollToTop();

  return {
    refresh: refreshAll,
    setLockUi,
  };
}
