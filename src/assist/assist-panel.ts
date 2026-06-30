import { saveAssistPrefs, loadAssistPrefs } from "../theme/theme-manager";
import { assistState, type AssistActionPreview } from "./assist-state";
import {
  ASSIST_MODELS,
  SESSION_MODE_LABELS,
  TOOLS_LEVEL_LABELS,
  type AssistSessionMode,
  type AssistToolsLevel,
} from "./assist-types";
import { escapeHtml } from "../types";

function renderPreviewCard(preview: AssistActionPreview): HTMLElement {
  const card = document.createElement("article");
  card.className = "assist-preview glass-card";
  card.innerHTML = `
    <header class="assist-preview__header">
      <span class="assist-preview__badge">Preview</span>
      <code class="assist-preview__tool">${escapeHtml(preview.tool_name)}</code>
    </header>
    <p class="assist-preview__line">${escapeHtml(preview.collapsed_line || preview.after_summary)}</p>
    <details class="assist-preview__details">
      <summary>Technical details</summary>
      <pre>${escapeHtml(JSON.stringify(preview.arguments, null, 2))}</pre>
    </details>
    <div class="assist-preview__actions">
      <button type="button" class="btn btn--primary btn--small" data-action="confirm">Confirm</button>
      <button type="button" class="btn btn--small" data-action="cancel">Cancel</button>
    </div>
  `;

  card.querySelector<HTMLButtonElement>('[data-action="confirm"]')!.onclick = () => {
    assistState.addMessage("assistant", `Applied: ${preview.collapsed_line}`);
    assistState.setPreview(null);
  };
  card.querySelector<HTMLButtonElement>('[data-action="cancel"]')!.onclick = () => {
    assistState.setPreview(null);
  };
  return card;
}

function renderMessageBubble(message: { role: string; content: string }): HTMLElement {
  const bubble = document.createElement("article");
  bubble.className = `assist-message assist-message--${message.role} glass-card`;
  bubble.innerHTML = `<div class="assist-message__body">${escapeHtml(message.content)}</div>`;
  return bubble;
}

export function mountAssistPanel(container: HTMLElement): () => void {
  const root = document.createElement("div");
  root.className = "assist-panel";
  container.replaceChildren(root);

  const render = () => {
    const { messages, prefs, pendingPreview, isResponding } = assistState;
    root.innerHTML = `
      <header class="assist-panel__header">
        <img src="/assets/branding/brian_mark.svg" alt="" class="assist-panel__mark" width="32" height="32" />
        <div class="assist-panel__controls">
          <select class="assist-select" data-field="mode" aria-label="Session mode">
            ${(["cloud", "byok", "local"] as AssistSessionMode[])
              .map(
                (m) =>
                  `<option value="${m}" ${prefs.session_mode === m ? "selected" : ""}>${SESSION_MODE_LABELS[m]}</option>`,
              )
              .join("")}
          </select>
          <select class="assist-select" data-field="model" aria-label="Model">
            ${ASSIST_MODELS.map(
              (m) =>
                `<option value="${m.id}" ${prefs.model_id === m.id ? "selected" : ""}>${escapeHtml(m.label)}</option>`,
            ).join("")}
          </select>
          <select class="assist-select" data-field="tools" aria-label="Tools level">
            ${(["core", "full"] as AssistToolsLevel[])
              .map(
                (t) =>
                  `<option value="${t}" ${prefs.tools_level === t ? "selected" : ""}>${TOOLS_LEVEL_LABELS[t]}</option>`,
              )
              .join("")}
          </select>
          <button type="button" class="icon-btn" data-action="clear" title="Clear chat">⌫</button>
        </div>
      </header>
      <div class="assist-panel__messages" data-messages></div>
      <div class="assist-panel__composer">
        <textarea rows="2" placeholder="Ask Brian…" aria-label="Message Brian" data-input></textarea>
        <button type="button" class="btn btn--primary" data-action="send" ${isResponding ? "disabled" : ""}>
          ${isResponding ? "…" : "Send"}
        </button>
      </div>
    `;

    const messagesHost = root.querySelector<HTMLElement>("[data-messages]")!;
    if (messages.length === 0 && !pendingPreview) {
      messagesHost.innerHTML = `<p class="empty-state">Brian Assist is ready. MCP orchestrator wiring lands in Phase 4; UI matches mobile preview/confirm flow.</p>`;
    } else {
      messagesHost.replaceChildren();
      for (const msg of messages) {
        messagesHost.appendChild(renderMessageBubble(msg));
      }
      if (pendingPreview) {
        messagesHost.appendChild(renderPreviewCard(pendingPreview));
      }
    }

    messagesHost.scrollTop = messagesHost.scrollHeight;

    root.querySelector<HTMLSelectElement>('[data-field="mode"]')!.onchange = async (e) => {
      assistState.setPrefs({
        ...assistState.prefs,
        session_mode: (e.target as HTMLSelectElement).value as AssistSessionMode,
      });
      await saveAssistPrefs(assistState.prefs);
    };
    root.querySelector<HTMLSelectElement>('[data-field="model"]')!.onchange = async (e) => {
      assistState.setPrefs({
        ...assistState.prefs,
        model_id: (e.target as HTMLSelectElement).value,
      });
      await saveAssistPrefs(assistState.prefs);
    };
    root.querySelector<HTMLSelectElement>('[data-field="tools"]')!.onchange = async (e) => {
      assistState.setPrefs({
        ...assistState.prefs,
        tools_level: (e.target as HTMLSelectElement).value as AssistToolsLevel,
      });
      await saveAssistPrefs(assistState.prefs);
    };

    root.querySelector<HTMLButtonElement>('[data-action="clear"]')!.onclick = () => assistState.clear();

    const send = () => {
      const input = root.querySelector<HTMLTextAreaElement>("[data-input]")!;
      const text = input.value.trim();
      if (!text || assistState.isResponding) return;
      input.value = "";
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

    root.querySelector<HTMLButtonElement>('[data-action="send"]')!.onclick = send;
    root.querySelector<HTMLTextAreaElement>("[data-input]")!.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        send();
      }
    });
  };

  render();
  return assistState.subscribe(render);
}

export async function initAssistPrefs(): Promise<void> {
  const saved = await loadAssistPrefs<typeof assistState.prefs>();
  if (saved) assistState.setPrefs(saved);
}
