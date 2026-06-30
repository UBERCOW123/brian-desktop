import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { initAssistPrefs } from "./assist/assist-panel";
import { completeOAuthFromUrl, refreshSession, startAppleSignIn } from "./auth";
import { mountReact, rerenderReact } from "./react/mount";
import { AuthActions, TitlebarControls } from "./react/TitlebarControls";
import { createWorkbench, type Workbench } from "./shell/workbench";
import type { AccentPalette, ThemeMode } from "./theme/types";
import { getUiPrefs, loadThemeFromBackend, saveThemePref } from "./theme/theme-manager";
import type { HealthResponse, SessionInfo } from "./types";

let shell: Workbench | null = null;
let session: SessionInfo | null = null;

async function refreshApp(): Promise<void> {
  await shell?.refresh();
}

function renderTitlebar(): void {
  const host = document.querySelector<HTMLElement>("#titlebar-controls");
  if (!host || !session) return;

  const prefs = getUiPrefs();
  rerenderReact(
    host,
    <TitlebarControls
      themeMode={prefs.theme_mode}
      accent={prefs.accent_palette}
      onThemeModeChange={(mode: ThemeMode) => void saveThemePref({ theme_mode: mode })}
      onAccentChange={(accent: AccentPalette) => void saveThemePref({ accent_palette: accent })}
      authSlot={
        <AuthActions
          signedIn={session.signed_in}
          email={session.email}
          workspaceId={session.workspace_id}
          onSignIn={() => void startAppleSignIn()}
          onSignOut={async () => {
            await invoke("auth_sign_out");
            await boot();
          }}
        />
      }
    />,
  );
}

async function boot(): Promise<void> {
  const status = document.querySelector<HTMLElement>("#status")!;

  try {
    const health = await invoke<HealthResponse>("health_check");
    status.classList.toggle("hidden", health.ok);
    if (!health.ok) {
      status.innerHTML = `<p class="error">Backend unavailable</p>`;
    }

    session = await refreshSession(boot);
    const prefs = await loadThemeFromBackend();
    await initAssistPrefs();

    const titlebarHost = document.querySelector<HTMLElement>("#titlebar-controls")!;
    if (!titlebarHost.dataset.mounted) {
      mountReact(titlebarHost, null);
      titlebarHost.dataset.mounted = "true";
    }
    renderTitlebar();

    if (!shell) {
      const host = document.querySelector<HTMLElement>("#app-shell")!;
      shell = await createWorkbench(host);
    } else {
      await shell.refresh();
    }
    shell.setLockUi(prefs.layout_locked);
  } catch (err) {
    status.classList.remove("hidden");
    status.innerHTML = `<p class="error">Backend unavailable: ${String(err)}</p>`;
  }
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

window.addEventListener("core:refresh", () => {
  void refreshApp();
});

window.addEventListener("theme:changed", () => {
  renderTitlebar();
});

void wireDeepLinks();
void boot();
