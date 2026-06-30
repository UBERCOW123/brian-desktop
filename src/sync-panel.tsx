import { mountReact } from "./react/mount";
import { SyncPanelToolbar } from "./react/SyncPanelToolbar";

export function renderSyncPanel(container: HTMLElement): void {
  container.className = "sync-panel";
  mountReact(container, <SyncPanelToolbar />);
}
