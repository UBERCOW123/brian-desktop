import { escapeHtml } from "../types";

export function renderHomeHeader(options: {
  greeting: string;
  momentumLine: string;
  onQuickAdd?: (title: string) => Promise<void>;
}): HTMLElement {
  const now = new Date();
  const dateLabel = now.toLocaleDateString(undefined, {
    weekday: "long",
    month: "short",
    day: "numeric",
  });

  const root = document.createElement("section");
  root.className = "home-header";
  root.innerHTML = `
    <div class="home-header__content">
      <h2 class="home-header__greeting">${escapeHtml(options.greeting)}</h2>
      <p class="home-header__date">${escapeHtml(dateLabel)}</p>
      <p class="home-header__momentum">${escapeHtml(options.momentumLine)}</p>
    </div>
  `;

  if (options.onQuickAdd) {
    const form = document.createElement("form");
    form.className = "home-header__quick-add";
    form.innerHTML = `
      <input type="text" name="title" placeholder="Quick add task" aria-label="Quick add task" />
      <button type="submit" class="btn btn--primary btn--small">Add</button>
    `;
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      const input = form.querySelector<HTMLInputElement>('input[name="title"]')!;
      const title = input.value.trim();
      if (!title) return;
      input.value = "";
      await options.onQuickAdd!(title);
    });
    root.querySelector(".home-header__content")!.appendChild(form);
  }

  return root;
}

export function dashboardGreeting(email: string | null): string {
  const hour = new Date().getHours();
  const period = hour < 12 ? "Good morning" : hour < 17 ? "Good afternoon" : "Good evening";
  if (!email) return `${period}.`;
  const name = email.split("@")[0]?.split(".")[0] ?? "there";
  const capitalized = name.charAt(0).toUpperCase() + name.slice(1);
  return `${period}, ${capitalized}.`;
}
