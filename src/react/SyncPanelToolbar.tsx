import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { BrianButton } from "@ui";

export function SyncPanelToolbar() {
  const [log, setLog] = useState("Ready.");

  return (
    <div className="flex h-full flex-col gap-2 p-3">
      <div className="flex flex-wrap gap-2">
        <BrianButton
          variant="secondary"
          className="py-1 text-xs"
          onClick={async () => {
            try {
              setLog(JSON.stringify(await invoke("sync_drain"), null, 2));
            } catch (err) {
              setLog(String(err));
            }
          }}
        >
          Drain outbox
        </BrianButton>
        <BrianButton
          variant="secondary"
          className="py-1 text-xs"
          onClick={async () => {
            try {
              setLog(JSON.stringify(await invoke("sync_pull"), null, 2));
              window.dispatchEvent(new CustomEvent("core:refresh"));
            } catch (err) {
              setLog(String(err));
            }
          }}
        >
          Pull server
        </BrianButton>
      </div>
      <pre className="min-h-0 flex-1 overflow-auto rounded-md border border-[var(--border-card)] bg-[var(--surface-elevated)] p-2 text-[11px] text-[var(--text-secondary)]">
        {log}
      </pre>
    </div>
  );
}
