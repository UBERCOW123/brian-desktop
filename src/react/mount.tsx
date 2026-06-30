import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";

const roots = new Map<HTMLElement, Root>();

export function mountReact(host: HTMLElement, element: ReactNode): () => void {
  let root = roots.get(host);
  if (!root) {
    root = createRoot(host);
    roots.set(host, root);
  }
  root.render(element);
  return () => {
    root?.unmount();
    roots.delete(host);
  };
}

export function rerenderReact(host: HTMLElement, element: ReactNode): void {
  roots.get(host)?.render(element);
}
