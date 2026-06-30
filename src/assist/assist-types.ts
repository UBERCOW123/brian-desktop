export type AssistSessionMode = "cloud" | "byok" | "local";
export type AssistToolsLevel = "core" | "full";

export const ASSIST_MODELS = [
  { id: "anthropic/claude-sonnet-4", label: "Claude Sonnet 4" },
  { id: "anthropic/claude-3.5-haiku", label: "Claude Haiku 3.5" },
  { id: "openai/gpt-4o-mini", label: "GPT-4o Mini" },
  { id: "google/gemini-2.0-flash-001", label: "Gemini 2.0 Flash" },
] as const;

export const SESSION_MODE_LABELS: Record<AssistSessionMode, string> = {
  cloud: "Cloud",
  byok: "BYOK",
  local: "Local",
};

export const TOOLS_LEVEL_LABELS: Record<AssistToolsLevel, string> = {
  core: "CORE tools",
  full: "Full tools",
};
