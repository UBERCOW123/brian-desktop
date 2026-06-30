import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { BrianBadge, BrianButton, BrianCard } from "@ui";
import type { WidgetCatalog, WidgetCatalogEntry, WidgetInstance } from "../types";

type MarketplacePanelProps = {
  onInstalled: () => Promise<void>;
};

export function MarketplacePanel({ onInstalled }: MarketplacePanelProps) {
  const [catalog, setCatalog] = useState<WidgetCatalog | null>(null);
  const [installed, setInstalled] = useState<WidgetInstance[]>([]);
  const [category, setCategory] = useState("");
  const [installing, setInstalling] = useState<string | null>(null);

  const load = useCallback(async () => {
    const [cat, widgets] = await Promise.all([
      invoke<WidgetCatalog>("widget_catalog"),
      invoke<WidgetInstance[]>("list_widgets"),
    ]);
    setCatalog(cat);
    setInstalled(widgets);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const categories = catalog
    ? [...new Set(catalog.widgets.map((w) => w.category).filter(Boolean))]
    : [];

  const filtered =
    catalog?.widgets.filter((w) => !category || w.category === category) ?? [];

  const install = async (entry: WidgetCatalogEntry) => {
    setInstalling(entry.widgetType);
    try {
      await invoke("install_widget", { widgetType: entry.widgetType });
      await load();
      await onInstalled();
    } catch (err) {
      alert(String(err));
    } finally {
      setInstalling(null);
    }
  };

  if (!catalog) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-[var(--text-secondary)]">
        Loading marketplace…
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="flex flex-wrap gap-2">
        <BrianButton
          variant={category === "" ? "primary" : "secondary"}
          className="py-1 text-xs"
          onClick={() => setCategory("")}
        >
          All
        </BrianButton>
        {categories.map((c) => (
          <BrianButton
            key={c}
            variant={category === c ? "primary" : "secondary"}
            className="py-1 text-xs"
            onClick={() => setCategory(c!)}
          >
            {c}
          </BrianButton>
        ))}
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
        {filtered.map((entry) => {
          const installedCount = installed.filter((w) => w.widget_type === entry.widgetType).length;
          const canInstall = entry.allowMultiple || installedCount === 0;
          const size = entry.defaultSize ?? { width: 2, height: 2 };
          const busy = installing === entry.widgetType;

          return (
            <BrianCard key={entry.widgetType} glass="surface" className="flex flex-col gap-3 p-4">
              <div>
                <h4 className="text-sm font-semibold text-[var(--text-primary)]">
                  {entry.displayName}
                </h4>
                <p className="mt-1 text-xs text-[var(--text-secondary)]">
                  {entry.widgetType} · {size.width}×{size.height}
                </p>
              </div>
              {entry.category ? <BrianBadge variant="neutral">{entry.category}</BrianBadge> : null}
              <BrianButton
                variant={canInstall ? "secondary" : "secondary"}
                className="mt-auto w-full py-1.5 text-xs"
                disabled={!canInstall || busy}
                isLoading={busy}
                loadingText="Installing…"
                onClick={() => void install(entry)}
              >
                {canInstall ? "Install" : "Installed"}
              </BrianButton>
            </BrianCard>
          );
        })}
      </div>
    </div>
  );
}
