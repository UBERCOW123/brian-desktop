import { invoke } from "@tauri-apps/api/core";
import type { SessionInfo, TaskQueueItem, TimelineEntry } from "./types";
import { escapeHtml, formatDate } from "./types";
import { createGlassCard } from "./components/glass-card";
import { dashboardGreeting, renderHomeHeader } from "./components/home-header";

function momentumLine(tasks: TaskQueueItem[]): string {
  const active = tasks.filter((t) => t.status === "active").length;
  const overdue = tasks.filter((t) => t.is_overdue).length;
  if (active === 0) return "No open tasks — add one from the marketplace.";
  if (overdue > 0) return `${active} active · ${overdue} overdue`;
  return `${active} active task${active === 1 ? "" : "s"}`;
}

export async function renderTimeline(container: HTMLElement): Promise<void> {
  container.className = "timeline-panel";

  const [session, tasks] = await Promise.all([
    invoke<SessionInfo>("auth_session_info"),
    invoke<TaskQueueItem[]>("list_tasks", { includeCompleted: false }).catch(
      () => [] as TaskQueueItem[],
    ),
  ]);

  const header = renderHomeHeader({
    greeting: dashboardGreeting(session.email),
    momentumLine: momentumLine(tasks),
    onQuickAdd: async (title) => {
      await invoke("create_task", { title });
      window.dispatchEvent(new CustomEvent("core:refresh"));
    },
  });
  const headerHost = document.createElement("div");
  headerHost.className = "home-header-host";
  headerHost.appendChild(header);

  const entries = await invoke<TimelineEntry[]>("list_timeline");

  if (entries.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No timeline entries yet. Tasks and milestones appear here.";
    container.replaceChildren(headerHost, empty);
    return;
  }

  const today = new Date().toDateString();
  const spine = document.createElement("div");
  spine.className = "timeline-spine";
  spine.setAttribute("aria-hidden", "true");
  spine.innerHTML = `<div class="timeline-spine__line"></div>`;

  const list = document.createElement("div");
  list.className = "timeline-spine-list";

  entries.forEach((entry, index) => {
    const isToday = new Date(entry.date).toDateString() === today;
    const side = index % 2 === 0 ? "left" : "right";

    const row = document.createElement("div");
    row.className = `timeline-spine-row timeline-spine-row--${side}`;

    const dotCol = document.createElement("div");
    dotCol.className = "timeline-spine-row__spine";
    dotCol.innerHTML = `
      <span class="timeline-spine-row__dot ${entry.is_critical ? "critical" : ""} ${entry.is_completed ? "done" : ""}"></span>
      <span class="timeline-spine-row__arm"></span>
    `;

    const cardInner = document.createElement("div");
    cardInner.innerHTML = `
      <div class="timeline-card__meta">
        <span class="timeline-card__type">${escapeHtml(entry.entry_type)}</span>
        ${isToday ? `<span class="today-pill">Today</span>` : ""}
        <time>${escapeHtml(formatDate(entry.date))}</time>
      </div>
      <h3 class="timeline-card__title">${escapeHtml(entry.title)}</h3>
      ${entry.subtitle ? `<p class="timeline-card__subtitle">${escapeHtml(entry.subtitle)}</p>` : ""}
    `;

    const card = createGlassCard(cardInner, {
      depth: "elevated",
      className: `timeline-card ${entry.is_critical ? "timeline-card--critical" : ""} ${entry.is_completed ? "timeline-card--done" : ""}`,
      padding: "0",
    });

    row.append(dotCol, card);
    list.appendChild(row);
  });

  container.replaceChildren(headerHost, spine, list);
}
