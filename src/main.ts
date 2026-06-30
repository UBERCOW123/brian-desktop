import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { open } from "@tauri-apps/plugin-shell";

interface HealthResponse {
  ok: boolean;
  schema_version: number;
  contracts_present: boolean;
  supabase_configured: boolean;
  signed_in: boolean;
}

interface SessionInfo {
  signed_in: boolean;
  user_id: string | null;
  email: string | null;
  workspace_id: string | null;
}

interface TaskQueueItem {
  record_id: string;
  title: string;
  display_title: string;
  status: "active" | "completed";
  is_overdue: boolean;
}

interface DrainReport {
  processed: number;
  synced: number;
  failed: number;
  errors: string[];
}

interface PullResult {
  records_pulled: number;
  links_pulled: number;
  events_pulled: number;
  errors: string[];
}

function el<T extends HTMLElement>(selector: string): T {
  return document.querySelector(selector)!;
}

async function refreshSession() {
  const session = await invoke<SessionInfo>("auth_session_info");
  const sessionCard = el<HTMLElement>("#session");
  const authActions = el<HTMLElement>("#auth-actions");

  if (session.signed_in) {
    sessionCard.innerHTML = `
      <h2>Session</h2>
      <dl>
        <dt>User</dt><dd><code>${session.email ?? session.user_id ?? "—"}</code></dd>
        <dt>Workspace</dt><dd><code>${session.workspace_id ?? "—"}</code></dd>
      </dl>
    `;
    authActions.innerHTML = `<button id="sign-out" type="button">Sign out</button>`;
    el<HTMLButtonElement>("#sign-out").onclick = async () => {
      await invoke("auth_sign_out");
      await boot();
    };
  } else {
    sessionCard.innerHTML = `<p>Not signed in. Use Apple ID to sync with mobile CORE.</p>`;
    authActions.innerHTML = `<button id="sign-in" type="button">Sign in with Apple</button>`;
    el<HTMLButtonElement>("#sign-in").onclick = startAppleSignIn;
  }
}

async function startAppleSignIn() {
  const { authorize_url } = await invoke<{ authorize_url: string }>("auth_start_apple");
  await open(authorize_url);
}

async function completeOAuthFromUrl(url: string) {
  if (!url.includes("login-callback")) {
    return;
  }
  await invoke("auth_complete_oauth", { callbackUrl: url });
  await boot();
}

async function refreshTasks() {
  const tasks = await invoke<TaskQueueItem[]>("list_tasks", { includeCompleted: false });
  const list = el<HTMLUListElement>("#task-list");
  if (tasks.length === 0) {
    list.innerHTML = `<li class="empty">No active tasks yet.</li>`;
    return;
  }
  list.innerHTML = tasks
    .map(
      (t) =>
        `<li><strong>${escapeHtml(t.display_title || t.title)}</strong>
         <span class="meta">${t.status}${t.is_overdue ? " · overdue" : ""}</span></li>`,
    )
    .join("");
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

async function boot() {
  const status = el<HTMLElement>("#status");
  const contracts = el<HTMLElement>("#contracts");

  try {
    const health = await invoke<HealthResponse>("health_check");
    status.innerHTML = `
      <h2>Status</h2>
      <ul>
        <li>OK: <strong>${health.ok}</strong></li>
        <li>SQLite schema v<strong>${health.schema_version}</strong></li>
        <li>Contracts: <strong>${health.contracts_present ? "present" : "missing"}</strong></li>
        <li>Supabase: <strong>${health.supabase_configured ? "configured" : "not configured"}</strong></li>
        <li>Signed in: <strong>${health.signed_in ? "yes" : "no"}</strong></li>
      </ul>
    `;

    const info = await invoke<{
      contracts_version_file: string;
      vendor_root: string;
      db_path: string;
    }>("contracts_info");
    contracts.innerHTML = `
      <h2>Contracts</h2>
      <dl>
        <dt>Pin</dt><dd><code>${info.contracts_version_file}</code></dd>
        <dt>Vendor</dt><dd><code>${info.vendor_root}</code></dd>
        <dt>DB</dt><dd><code>${info.db_path}</code></dd>
      </dl>
    `;

    await refreshSession();
    await refreshTasks();
  } catch (err) {
    status.innerHTML = `<p class="error">Backend unavailable: ${String(err)}</p>`;
  }
}

function wireForms() {
  el<HTMLFormElement>("#task-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const input = el<HTMLInputElement>("#task-title");
    const title = input.value.trim();
    if (!title) return;
    await invoke("create_task", { title });
    input.value = "";
    await refreshTasks();
  });

  el<HTMLButtonElement>("#sync-drain").onclick = async () => {
    const log = el<HTMLElement>("#sync-log");
    try {
      const report = await invoke<DrainReport>("sync_drain");
      log.textContent = JSON.stringify(report, null, 2);
    } catch (err) {
      log.textContent = String(err);
    }
  };

  el<HTMLButtonElement>("#sync-pull").onclick = async () => {
    const log = el<HTMLElement>("#sync-log");
    try {
      const report = await invoke<PullResult>("sync_pull");
      log.textContent = JSON.stringify(report, null, 2);
      await refreshTasks();
    } catch (err) {
      log.textContent = String(err);
    }
  };
}

async function wireDeepLinks() {
  const current = await getCurrent();
  if (current?.length) {
    for (const url of current) {
      await completeOAuthFromUrl(url);
    }
  }

  await onOpenUrl((urls) => {
    void (async () => {
      for (const url of urls) {
        await completeOAuthFromUrl(url);
      }
    })();
  });
}

wireForms();
void wireDeepLinks();
void boot();
