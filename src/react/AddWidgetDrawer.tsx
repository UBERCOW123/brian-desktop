import { MarketplacePanel } from "./MarketplacePanel";

type AddWidgetDrawerProps = {
  open: boolean;
  onClose: () => void;
  onInstalled: () => Promise<void>;
};

export function AddWidgetDrawer({ open, onClose, onInstalled }: AddWidgetDrawerProps) {
  return (
    <>
      <button
        type="button"
        className={`workbench-drawer-backdrop ${open ? "workbench-drawer-backdrop--open" : ""}`}
        aria-label="Close add widget drawer"
        onClick={onClose}
      />
      <aside
        className={`workbench-drawer ${open ? "workbench-drawer--open" : ""}`}
        aria-hidden={!open}
        aria-label="Add widget"
      >
        <header className="workbench-drawer__header">
          <h2 className="workbench-drawer__title">Add widget</h2>
          <button type="button" className="icon-btn" onClick={onClose} aria-label="Close">
            ×
          </button>
        </header>
        <div className="workbench-drawer__body">
          {open ? <MarketplacePanel onInstalled={onInstalled} /> : null}
        </div>
      </aside>
    </>
  );
}
