import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { completeOAuthFromUrl, refreshSession, startAppleSignIn } from "./auth";
import { renderDashboardGrid } from "./dashboard";
import { renderMarketplace } from "./marketplace";
import { renderTimeline } from "./timeline";
import type { HealthResponse } from "./types";

async function refreshDashboard(): Promise<void> {
  const gridHost = document.querySelector<HTMLElement>("#widget-grid")!;
  const marketplaceHost = document.querySelector<HTMLElement>("#marketplace")!;
  const widgets = await renderDashboardGrid(gridHost, refreshDashboard);
  await renderMarketplace(marketplaceHost, widgets, refreshDashboard);
}

async function boot(): Promise<void> {
  const status = document.querySelector<HTMLElement>("#status")!;
  const timeline = document.querySelector<HTMLElement>("#timeline")!;

  try {
    const health = await invoke<HealthResponse>("health_check");
    status.classList.toggle("hidden", health.ok);
    if (!health.ok) {
      status.innerHTML = `<p class="error">Backend unavailable</p>`;
    }

    await refreshSession(startAppleSignIn, boot);
    await renderTimeline(timeline);
    await refreshDashboard();
  } catch (err) {
    status.classList.remove("hidden");
    status.innerHTML = `<p class="error">Backend unavailable: ${String(err)}</p>`;
  }
}

function wireChrome(): void {
  const drawer = document.querySelector<HTMLElement>("#drawer")!;
  const openBtn = document.querySelector<HTMLButtonElement>("#open-drawer")!;
  const closeBtn = document.querySelector<HTMLButtonElement>("#close-drawer")!;
  const scrim = document.querySelector<HTMLElement>("#drawer-scrim")!;
  const tabs = document.querySelectorAll<HTMLButtonElement>(".drawer-tab");

  const setDrawer = (open: boolean) => {
    document.body.classList.toggle("drawer-open", open);
    drawer.setAttribute("aria-hidden", open ? "false" : "true");
    scrim.hidden = !open;
  };

  openBtn.onclick = () => setDrawer(true);
  closeBtn.onclick = () => setDrawer(false);
  scrim.onclick = () => setDrawer(false);

  tabs.forEach((tab) => {
    tab.onclick = () => {
      tabs.forEach((t) => t.classList.remove("active"));
      tab.classList.add("active");
      const panel = tab.dataset.panel!;
      document.querySelectorAll<HTMLElement>(".drawer-panel").forEach((p) => {
        p.classList.toggle("active", p.id === panel);
      });
    };
  });

  document.querySelector<HTMLButtonElement>("#sync-drain")!.onclick = async () => {
    const log = document.querySelector<HTMLElement>("#sync-log")!;
    try {
      log.textContent = JSON.stringify(await invoke("sync_drain"), null, 2);
    } catch (err) {
      log.textContent = String(err);
    }
  };

  document.querySelector<HTMLButtonElement>("#sync-pull")!.onclick = async () => {
    const log = document.querySelector<HTMLElement>("#sync-log")!;
    try {
      log.textContent = JSON.stringify(await invoke("sync_pull"), null, 2);
      await boot();
    } catch (err) {
      log.textContent = String(err);
    }
  };

  document.querySelector<HTMLFormElement>("#task-form")!.addEventListener("submit", async (event) => {
    event.preventDefault();
    const input = document.querySelector<HTMLInputElement>("#task-title")!;
    const title = input.value.trim();
    if (!title) return;
    await invoke("create_task", { title });
    input.value = "";
    await boot();
  });
}

async function wireDeepLinks(): Promise<void> {
  const current = await getCurrent();
  if (current?.length) {
    for (const url of current) {
      await completeOAuthFromUrl(url, boot);
    }
  }
  await onOpenUrl((urls) => {
    void (async () => {
      for (const url of urls) {
        await completeOAuthFromUrl(url, boot);
      }
    })();
  });
}

wireChrome();
void wireDeepLinks();
void boot();
