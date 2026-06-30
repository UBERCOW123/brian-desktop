export { mountDashboardGrid, DashboardGrid } from "./layout/dashboard-grid";

/** @deprecated Use mountDashboardGrid via widgets-workspace */
export async function renderDashboardGrid(
  container: HTMLElement,
  onLayoutSaved: () => Promise<void>,
): Promise<import("./types").WidgetInstance[]> {
  const { mountDashboardGrid } = await import("./layout/dashboard-grid");
  const grid = await mountDashboardGrid(container, onLayoutSaved);
  return grid.load();
}
