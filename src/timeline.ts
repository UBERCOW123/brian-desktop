import { invoke } from "@tauri-apps/api/core";
import type { TimelineEntry } from "./types";
import { escapeHtml, formatDate } from "./types";

export async function renderTimeline(container: HTMLElement): Promise<void> {
  const entries = await invoke<TimelineEntry[]>("list_timeline");
  if (entries.length === 0) {
    container.innerHTML = `<p class="empty-state">No timeline entries yet. Tasks and milestones appear here.</p>`;
    return;
  }

  const today = new Date().toDateString();
  container.innerHTML = entries
    .map((entry) => {
      const isToday = new Date(entry.date).toDateString() === today;
      return `
        <article class="timeline-entry ${entry.is_critical ? "critical" : ""} ${entry.is_completed ? "done" : ""}">
          <div class="timeline-marker" aria-hidden="true"></div>
          <div class="timeline-body">
            <div class="timeline-meta">
              <span class="timeline-type">${escapeHtml(entry.entry_type)}</span>
              ${isToday ? `<span class="today-pill">Today</span>` : ""}
              <time>${escapeHtml(formatDate(entry.date))}</time>
            </div>
            <h3>${escapeHtml(entry.title)}</h3>
            ${entry.subtitle ? `<p>${escapeHtml(entry.subtitle)}</p>` : ""}
          </div>
        </article>
      `;
    })
    .join("");
}
