import { invoke } from "@tauri-apps/api/core";
import type { SessionInfo } from "./types";
import { escapeHtml } from "./types";

export async function refreshSession(
  onSignIn: () => void,
  onBoot: () => Promise<void>,
): Promise<void> {
  const session = await invoke<SessionInfo>("auth_session_info");
  const sessionCard = document.querySelector<HTMLElement>("#session")!;
  const authActions = document.querySelector<HTMLElement>("#auth-actions")!;

  if (session.signed_in) {
    sessionCard.innerHTML = `
      <dl class="session-dl">
        <dt>User</dt><dd>${escapeHtml(session.email ?? session.user_id ?? "—")}</dd>
        <dt>Workspace</dt><dd><code>${escapeHtml(session.workspace_id ?? "—")}</code></dd>
      </dl>
    `;
    authActions.innerHTML = `<button id="sign-out" type="button" class="btn">Sign out</button>`;
    document.querySelector<HTMLButtonElement>("#sign-out")!.onclick = async () => {
      await invoke("auth_sign_out");
      await onBoot();
    };
  } else {
    sessionCard.innerHTML = `<p class="hint">Sign in with Apple to sync with mobile CORE.</p>`;
    authActions.innerHTML = `<button id="sign-in" type="button" class="btn btn-primary">Sign in</button>`;
    document.querySelector<HTMLButtonElement>("#sign-in")!.onclick = onSignIn;
  }
}

export async function startAppleSignIn(): Promise<void> {
  const { open } = await import("@tauri-apps/plugin-shell");
  const { authorize_url } = await invoke<{ authorize_url: string }>("auth_start_apple");
  await open(authorize_url);
}

export async function completeOAuthFromUrl(
  url: string,
  onBoot: () => Promise<void>,
): Promise<void> {
  if (!url.includes("login-callback")) return;
  await invoke("auth_complete_oauth", { callbackUrl: url });
  await onBoot();
}
