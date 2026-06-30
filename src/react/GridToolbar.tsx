import { BrianButton, BrianLabel, BrianSwitch } from "@ui";

type GridToolbarProps = {
  layoutLocked: boolean;
  drawerOpen: boolean;
  onLockChange: (locked: boolean) => void;
  onAddWidget: () => void;
  onResetLayout: () => void;
};

export function GridToolbar({
  layoutLocked,
  drawerOpen,
  onLockChange,
  onAddWidget,
  onResetLayout,
}: GridToolbarProps) {
  return (
    <div className="workbench-toolbar">
      <div className="workbench-toolbar__group">
        <BrianButton variant="secondary" className="workbench-toolbar__btn" onClick={onAddWidget}>
          {drawerOpen ? "Close" : "Add widget"}
        </BrianButton>
        <BrianButton variant="secondary" className="workbench-toolbar__btn" onClick={onResetLayout}>
          Reset layout
        </BrianButton>
      </div>
      <div className="workbench-toolbar__group workbench-toolbar__group--end">
        <BrianLabel htmlFor="layout-lock" className="workbench-toolbar__label">
          Lock layout
        </BrianLabel>
        <BrianSwitch
          id="layout-lock"
          checked={layoutLocked}
          onCheckedChange={onLockChange}
          aria-label="Lock widget layout"
        />
      </div>
    </div>
  );
}
