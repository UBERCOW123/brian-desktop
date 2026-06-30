import { useEffect, useSyncExternalStore } from "react";
import { saveAssistPrefs } from "../theme/theme-manager";
import { assistState, type AssistActionPreview } from "../assist/assist-state";
import {
  ASSIST_MODELS,
  SESSION_MODE_LABELS,
  TOOLS_LEVEL_LABELS,
  type AssistSessionMode,
  type AssistToolsLevel,
} from "../assist/assist-types";
import { BrianButton, BrianCard, BrianLabel, BrianSelect } from "@ui";

function useAssistState() {
  return useSyncExternalStore(
    (listener) => assistState.subscribe(listener),
    () => ({
      messages: assistState.messages,
      prefs: assistState.prefs,
      pendingPreview: assistState.pendingPreview,
      isResponding: assistState.isResponding,
    }),
  );
}

function PreviewCard({ preview }: { preview: AssistActionPreview }) {
  return (
    <BrianCard glass="elevated" className="flex flex-col gap-3 p-3">
      <header className="flex items-center gap-2">
        <span className="text-[10px] font-semibold uppercase tracking-wide text-[var(--accent)]">
          Preview
        </span>
        <code className="text-xs text-[var(--text-secondary)]">{preview.tool_name}</code>
      </header>
      <p className="text-sm text-[var(--text-primary)]">
        {preview.collapsed_line || preview.after_summary}
      </p>
      <details className="text-xs text-[var(--text-secondary)]">
        <summary>Technical details</summary>
        <pre className="mt-2 overflow-auto rounded-md bg-[var(--surface-elevated)] p-2">
          {JSON.stringify(preview.arguments, null, 2)}
        </pre>
      </details>
      <div className="flex gap-2">
        <BrianButton
          className="py-1 text-xs"
          onClick={() => {
            assistState.addMessage("assistant", `Applied: ${preview.collapsed_line}`);
            assistState.setPreview(null);
          }}
        >
          Confirm
        </BrianButton>
        <BrianButton variant="secondary" className="py-1 text-xs" onClick={() => assistState.setPreview(null)}>
          Cancel
        </BrianButton>
      </div>
    </BrianCard>
  );
}

function MessageBubble({ role, content }: { role: string; content: string }) {
  const isUser = role === "user";
  return (
    <BrianCard
      glass={isUser ? "surface" : "elevated"}
      className={`max-w-[95%] p-3 text-sm ${isUser ? "ml-auto" : "mr-auto"}`}
    >
      <div className="text-[var(--text-primary)]">{content}</div>
    </BrianCard>
  );
}

export function AssistWidget() {
  const { messages, prefs, pendingPreview, isResponding } = useAssistState();

  useEffect(() => {
    const host = document.querySelector<HTMLElement>("[data-assist-messages]");
    host?.scrollTo({ top: host.scrollHeight, behavior: "smooth" });
  }, [messages, pendingPreview]);

  const send = () => {
    const input = document.querySelector<HTMLTextAreaElement>("[data-assist-input]");
    const text = input?.value.trim();
    if (!text || assistState.isResponding) return;
    if (input) input.value = "";
    assistState.addMessage("user", text);

    assistState.isResponding = true;
    assistState.notify();
    window.setTimeout(() => {
      assistState.isResponding = false;
      if (text.toLowerCase().includes("create task")) {
        assistState.setPreview({
          operation_id: crypto.randomUUID(),
          tool_name: "tasks.create",
          collapsed_line: `Create task "${text.replace(/^create task:?\s*/i, "") || "New task"}"`,
          after_summary: "A new task will appear in your timeline after confirm.",
          arguments: { displayTitle: text.replace(/^create task:?\s*/i, "") || "New task" },
        });
      } else {
        assistState.addMessage(
          "assistant",
          "I'm Brian. Full MCP orchestration connects in Phase 4 — try “create task: …” to see preview/confirm.",
        );
      }
      assistState.notify();
    }, 600);
  };

  return (
    <div className="assist-widget flex h-full min-h-0 flex-col">
      <header className="assist-widget__header flex shrink-0 flex-wrap items-end gap-2 px-1 pb-2">
        <img
          src="/assets/branding/brian_mark.svg"
          alt=""
          width={24}
          height={24}
          className="titlebar__mark shrink-0 self-center"
        />
        <div className="grid min-w-0 flex-1 grid-cols-3 gap-2">
          <div>
            <BrianLabel htmlFor="assist-mode" className="text-[10px] text-[var(--text-tertiary)]">
              Mode
            </BrianLabel>
            <BrianSelect
              id="assist-mode"
              className="mt-0.5 w-full py-1 text-xs"
              value={prefs.session_mode}
              onChange={async (event) => {
                assistState.setPrefs({
                  ...assistState.prefs,
                  session_mode: event.target.value as AssistSessionMode,
                });
                await saveAssistPrefs(assistState.prefs);
              }}
            >
              {(["cloud", "byok", "local"] as AssistSessionMode[]).map((mode) => (
                <option key={mode} value={mode}>
                  {SESSION_MODE_LABELS[mode]}
                </option>
              ))}
            </BrianSelect>
          </div>
          <div>
            <BrianLabel htmlFor="assist-model" className="text-[10px] text-[var(--text-tertiary)]">
              Model
            </BrianLabel>
            <BrianSelect
              id="assist-model"
              className="mt-0.5 w-full py-1 text-xs"
              value={prefs.model_id}
              onChange={async (event) => {
                assistState.setPrefs({ ...assistState.prefs, model_id: event.target.value });
                await saveAssistPrefs(assistState.prefs);
              }}
            >
              {ASSIST_MODELS.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.label}
                </option>
              ))}
            </BrianSelect>
          </div>
          <div>
            <BrianLabel htmlFor="assist-tools" className="text-[10px] text-[var(--text-tertiary)]">
              Tools
            </BrianLabel>
            <BrianSelect
              id="assist-tools"
              className="mt-0.5 w-full py-1 text-xs"
              value={prefs.tools_level}
              onChange={async (event) => {
                assistState.setPrefs({
                  ...assistState.prefs,
                  tools_level: event.target.value as AssistToolsLevel,
                });
                await saveAssistPrefs(assistState.prefs);
              }}
            >
              {(["core", "full"] as AssistToolsLevel[]).map((level) => (
                <option key={level} value={level}>
                  {TOOLS_LEVEL_LABELS[level]}
                </option>
              ))}
            </BrianSelect>
          </div>
        </div>
        <BrianButton variant="secondary" className="shrink-0 self-end py-1 text-xs" onClick={() => assistState.clear()}>
          Clear
        </BrianButton>
      </header>

      <div
        data-assist-messages
        className="flex min-h-0 flex-1 flex-col gap-2 overflow-auto px-1 py-2"
      >
        {messages.length === 0 && !pendingPreview ? (
          <p className="empty-state text-center text-sm text-[var(--text-secondary)]">
            Brian Assist is ready. MCP orchestrator wiring lands in Phase 4; UI matches mobile preview/confirm flow.
          </p>
        ) : (
          messages.map((message) => (
            <MessageBubble key={message.id} role={message.role} content={message.content} />
          ))
        )}
        {pendingPreview ? <PreviewCard preview={pendingPreview} /> : null}
      </div>

      <div className="assist-widget__composer flex shrink-0 gap-2 pt-2">
        <textarea
          data-assist-input
          rows={2}
          placeholder="Ask Brian…"
          aria-label="Message Brian"
          className="brian-field min-h-[2.5rem] flex-1 resize-none rounded-md border px-3 py-2 text-sm"
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              send();
            }
          }}
        />
        <BrianButton className="self-end py-2 text-xs" disabled={isResponding} onClick={send}>
          {isResponding ? "…" : "Send"}
        </BrianButton>
      </div>
    </div>
  );
}
