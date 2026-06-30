import type { AssistSessionMode, AssistToolsLevel } from "./assist-types";

export interface AssistPrefs {
  session_mode: AssistSessionMode;
  model_id: string;
  tools_level: AssistToolsLevel;
}

export const DEFAULT_ASSIST_PREFS: AssistPrefs = {
  session_mode: "cloud",
  model_id: "anthropic/claude-sonnet-4",
  tools_level: "core",
};

export interface AssistMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
}

export interface AssistActionPreview {
  operation_id: string;
  tool_name: string;
  collapsed_line: string;
  after_summary: string;
  arguments: Record<string, unknown>;
}

export class AssistState {
  messages: AssistMessage[] = [];
  prefs: AssistPrefs = { ...DEFAULT_ASSIST_PREFS };
  pendingPreview: AssistActionPreview | null = null;
  isResponding = false;

  private listeners = new Set<() => void>();

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  notify(): void {
    this.listeners.forEach((l) => l());
  }

  setPrefs(prefs: AssistPrefs): void {
    this.prefs = prefs;
    this.notify();
  }

  addMessage(role: AssistMessage["role"], content: string): void {
    this.messages.push({
      id: crypto.randomUUID(),
      role,
      content,
      timestamp: Date.now(),
    });
    this.notify();
  }

  setPreview(preview: AssistActionPreview | null): void {
    this.pendingPreview = preview;
    this.notify();
  }

  clear(): void {
    this.messages = [];
    this.pendingPreview = null;
    this.notify();
  }
}

export const assistState = new AssistState();
