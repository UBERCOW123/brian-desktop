import { invoke } from "@tauri-apps/api/core";

interface HealthResponse {
  ok: boolean;
  schema_version: number;
  contracts_present: boolean;
}

interface ContractsInfo {
  vendor_root: string;
  contracts_version_file: string;
  sync_strategy: string;
  data_model: string;
  widget_seed: string;
  db_path: string;
}

async function boot() {
  const status = document.querySelector<HTMLElement>("#status")!;
  const contracts = document.querySelector<HTMLElement>("#contracts")!;

  try {
    const health = await invoke<HealthResponse>("health_check");
    status.innerHTML = `
      <h2>Status</h2>
      <ul>
        <li>OK: <strong>${health.ok}</strong></li>
        <li>SQLite schema v<strong>${health.schema_version}</strong></li>
        <li>Contracts: <strong>${health.contracts_present ? "present" : "missing"}</strong></li>
      </ul>
    `;

    const info = await invoke<ContractsInfo>("contracts_info");
    contracts.innerHTML = `
      <h2>Contracts</h2>
      <dl>
        <dt>Pin</dt><dd><code>${info.contracts_version_file}</code></dd>
        <dt>Vendor</dt><dd><code>${info.vendor_root}</code></dd>
        <dt>DB</dt><dd><code>${info.db_path}</code></dd>
      </dl>
      <p class="hint">Dashboard, sync, and Work Room land in the implementation plan.</p>
    `;
  } catch (err) {
    status.innerHTML = `<p class="error">Backend unavailable: ${String(err)}</p>`;
  }
}

void boot();
