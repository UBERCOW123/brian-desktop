import { useCallback, useEffect, useMemo, useState } from "react";
import { getBrowserState, setBrowserState, type BrowserState } from "../api/companion";
import { BrianButton, BrianInput } from "@ui";

const DEFAULT_URL = "https://example.com";

function isHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

function normalizeUrl(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return DEFAULT_URL;
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) return trimmed;
  return `https://${trimmed}`;
}

export function BrowserWidget() {
  const [state, setState] = useState<BrowserState>({
    url: DEFAULT_URL,
    history: [DEFAULT_URL],
    history_index: 0,
  });
  const [address, setAddress] = useState(DEFAULT_URL);
  const [frameUrl, setFrameUrl] = useState(DEFAULT_URL);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        const saved = await getBrowserState();
        const url = isHttpUrl(saved.url) ? saved.url : DEFAULT_URL;
        const history = saved.history.length > 0 ? saved.history : [url];
        const historyIndex = Math.min(Math.max(saved.history_index, 0), history.length - 1);
        const next = { url: history[historyIndex] ?? url, history, history_index: historyIndex };
        setState(next);
        setAddress(next.url);
        setFrameUrl(next.url);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const persist = useCallback(async (next: BrowserState) => {
    setState(next);
    setAddress(next.url);
    setFrameUrl(next.url);
    await setBrowserState(next);
  }, []);

  const navigate = useCallback(
    async (raw: string) => {
      setError(null);
      const nextUrl = normalizeUrl(raw);
      if (!isHttpUrl(nextUrl)) {
        setError("Only http and https URLs are allowed");
        return;
      }
      const trimmedHistory = state.history.slice(0, state.history_index + 1);
      trimmedHistory.push(nextUrl);
      await persist({
        url: nextUrl,
        history: trimmedHistory,
        history_index: trimmedHistory.length - 1,
      });
    },
    [persist, state.history, state.history_index],
  );

  const canGoBack = state.history_index > 0;
  const canGoForward = state.history_index < state.history.length - 1;

  const goBack = async () => {
    if (!canGoBack) return;
    const historyIndex = state.history_index - 1;
    const url = state.history[historyIndex] ?? DEFAULT_URL;
    await persist({ ...state, url, history_index: historyIndex });
  };

  const goForward = async () => {
    if (!canGoForward) return;
    const historyIndex = state.history_index + 1;
    const url = state.history[historyIndex] ?? DEFAULT_URL;
    await persist({ ...state, url, history_index: historyIndex });
  };

  const reload = () => {
    setFrameUrl("");
    window.setTimeout(() => setFrameUrl(state.url), 0);
  };

  const toolbar = useMemo(
    () => (
      <div className="companion-toolbar">
        <BrianButton variant="secondary" className="py-1 text-xs" disabled={!canGoBack} onClick={() => void goBack()}>
          Back
        </BrianButton>
        <BrianButton
          variant="secondary"
          className="py-1 text-xs"
          disabled={!canGoForward}
          onClick={() => void goForward()}
        >
          Forward
        </BrianButton>
        <BrianButton variant="secondary" className="py-1 text-xs" onClick={reload}>
          Reload
        </BrianButton>
        <form
          className="companion-toolbar__form"
          onSubmit={(event) => {
            event.preventDefault();
            void navigate(address);
          }}
        >
          <BrianInput
            value={address}
            onChange={(event) => setAddress(event.target.value)}
            className="py-1 text-xs"
            aria-label="Address"
            placeholder="https://…"
          />
          <BrianButton className="py-1 text-xs" type="submit">
            Go
          </BrianButton>
        </form>
      </div>
    ),
    [address, canGoBack, canGoForward, navigate, state],
  );

  if (loading) {
    return <p className="widget-empty">Loading browser…</p>;
  }

  return (
    <div className="browser-widget flex h-full min-h-0 flex-col gap-2">
      {toolbar}
      {error ? <p className="text-xs text-[var(--danger)]">{error}</p> : null}
      <iframe
        key={frameUrl}
        title="Browser"
        src={frameUrl}
        className="browser-widget__frame min-h-0 flex-1 rounded-md border border-[var(--border-subtle)] bg-white"
        sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
      />
    </div>
  );
}
