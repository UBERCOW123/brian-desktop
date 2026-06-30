import { useCallback, useEffect, useState } from "react";
import {
  createNote,
  listNotes,
  updateNote,
  type NotebookNote,
} from "../api/companion";
import { renderMarkdownPreview } from "../utils/markdown-preview";
import { BrianButton, BrianInput, BrianLabel } from "@ui";

export function NotebookWidget() {
  const [notes, setNotes] = useState<NotebookNote[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadNotes = useCallback(async () => {
    const items = await listNotes();
    setNotes(items);
    return items;
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const items = await loadNotes();
        if (items[0]) {
          setSelectedId(items[0].id);
          setTitle(items[0].title);
          setBody(items[0].body);
        }
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    })();
  }, [loadNotes]);

  const selectNote = (note: NotebookNote) => {
    setSelectedId(note.id);
    setTitle(note.title);
    setBody(note.body);
    setError(null);
  };

  const handleCreate = async () => {
    setSaving(true);
    setError(null);
    try {
      const id = await createNote("Untitled note", "");
      await loadNotes();
      setSelectedId(id);
      setTitle("Untitled note");
      setBody("");
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleSave = async () => {
    if (!selectedId) return;
    setSaving(true);
    setError(null);
    try {
      await updateNote(selectedId, title.trim() || "Untitled note", body);
      await loadNotes();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <p className="widget-empty">Loading notebook…</p>;
  }

  return (
    <div className="notebook-widget flex h-full min-h-0 gap-2">
      <aside className="notebook-widget__sidebar flex w-36 shrink-0 flex-col gap-2">
        <BrianButton className="py-1 text-xs" disabled={saving} onClick={() => void handleCreate()}>
          New note
        </BrianButton>
        <div className="min-h-0 flex-1 overflow-auto">
          {notes.length === 0 ? (
            <p className="text-xs text-[var(--text-secondary)]">No notes yet</p>
          ) : (
            <ul className="notebook-widget__list">
              {notes.map((note) => (
                <li key={note.id}>
                  <button
                    type="button"
                    className={`notebook-widget__item ${note.id === selectedId ? "notebook-widget__item--active" : ""}`}
                    onClick={() => selectNote(note)}
                  >
                    {note.title || "Untitled note"}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </aside>

      <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-2">
        <div className="flex items-end gap-2">
          <div className="min-w-0 flex-1">
            <BrianLabel htmlFor="notebook-title" className="text-[10px] text-[var(--text-tertiary)]">
              Title
            </BrianLabel>
            <BrianInput
              id="notebook-title"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              className="mt-0.5 py-1 text-xs"
            />
          </div>
          <BrianButton className="py-1 text-xs" disabled={!selectedId || saving} onClick={() => void handleSave()}>
            {saving ? "Saving…" : "Save"}
          </BrianButton>
        </div>
        {error ? <p className="text-xs text-[var(--danger)]">{error}</p> : null}
        <div className="notebook-widget__split min-h-0 flex-1">
          <textarea
            value={body}
            onChange={(event) => setBody(event.target.value)}
            className="notebook-widget__editor brian-field min-h-0 flex-1 resize-none rounded-md border p-2 text-xs"
            placeholder="Write markdown…"
            aria-label="Note editor"
          />
          <div
            className="notebook-widget__preview min-h-0 flex-1 overflow-auto rounded-md border border-[var(--border-subtle)] p-2 text-xs"
            dangerouslySetInnerHTML={{ __html: renderMarkdownPreview(body) }}
          />
        </div>
      </div>
    </div>
  );
}
