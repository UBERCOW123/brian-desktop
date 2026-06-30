import { invoke } from "@tauri-apps/api/core";
import type { SessionInfo } from "./types";

export async function refreshSession(_onBoot: () => Promise<void>): Promise<SessionInfo> {
  return invoke<SessionInfo>("auth_session_info");
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
