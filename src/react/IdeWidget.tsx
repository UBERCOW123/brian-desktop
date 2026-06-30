import Editor from "@monaco-editor/react";
import { Command } from "@tauri-apps/plugin-shell";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { useEffect, useRef, useState } from "react";
import {
  getIdeState,
  listIdeFiles,
  pickIdeFolder,
  readIdeFile,
  setIdeState,
  writeIdeFile,
  type IdeFileEntry,
  type IdeState,
} from "../api/companion";
import { BrianButton } from "@ui";
import "@xterm/xterm/css/xterm.css";

export function IdeWidget() {
  const [state, setState] = useState<IdeState>({ root_path: null, open_file: null });
  const [files, setFiles] = useState<IdeFileEntry[]>([]);
  const [editorValue, setEditorValue] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const terminalHostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        const saved = await getIdeState();
        setState(saved);
        if (saved.root_path) {
          const entries = await listIdeFiles();
          setFiles(entries.filter((entry) => !entry.is_dir));
          if (saved.open_file) {
            const contents = await readIdeFile(saved.open_file);
            setEditorValue(contents);
          }
        }
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  useEffect(() => {
    if (!terminalHostRef.current || !state.root_path || terminalRef.current) return;

    const terminal = new Terminal({
      fontSize: 12,
      theme: { background: "#0f1419", foreground: "#e6edf3" },
      cursorBlink: true,
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(terminalHostRef.current);
    fit.fit();
    terminalRef.current = terminal;
    fitRef.current = fit;

    const shell = Command.create("powershell", ["-NoLogo"], { cwd: state.root_path });
    shell.stdout.on("data", (line: string) => terminal.write(line));
    shell.stderr.on("data", (line: string) => terminal.write(line));
    void (async () => {
      try {
        const child = await shell.spawn();
        terminal.onData((data) => {
          void child.write(data);
        });
      } catch (err) {
        terminal.writeln(`Terminal unavailable: ${String(err)}`);
      }
    })();

    const onResize = () => fit.fit();
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("resize", onResize);
      terminal.dispose();
      terminalRef.current = null;
      fitRef.current = null;
    };
  }, [state.root_path]);

  const openFile = async (path: string) => {
    setError(null);
    try {
      const contents = await readIdeFile(path);
      setEditorValue(contents);
      const next = { ...state, open_file: path };
      setState(next);
      await setIdeState(next);
    } catch (err) {
      setError(String(err));
    }
  };

  const saveFile = async () => {
    if (!state.open_file) return;
    setSaving(true);
    setError(null);
    try {
      await writeIdeFile(state.open_file, editorValue);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const chooseFolder = async () => {
    setError(null);
    try {
      const picked = await pickIdeFolder();
      if (!picked) return;
      const next = { root_path: picked, open_file: null };
      setState(next);
      setEditorValue("");
      const entries = await listIdeFiles();
      setFiles(entries.filter((entry) => !entry.is_dir));
    } catch (err) {
      setError(String(err));
    }
  };

  if (loading) {
    return <p className="widget-empty">Loading IDE…</p>;
  }

  return (
    <div className="ide-widget flex h-full min-h-0 flex-col gap-2">
      <div className="companion-toolbar">
        <BrianButton variant="secondary" className="py-1 text-xs" onClick={() => void chooseFolder()}>
          Open folder
        </BrianButton>
        <BrianButton className="py-1 text-xs" disabled={!state.open_file || saving} onClick={() => void saveFile()}>
          {saving ? "Saving…" : "Save file"}
        </BrianButton>
        <span className="truncate text-xs text-[var(--text-secondary)]">
          {state.root_path ?? "No folder selected"}
        </span>
      </div>
      {error ? <p className="text-xs text-[var(--danger)]">{error}</p> : null}
      <div className="ide-widget__body min-h-0 flex-1">
        <aside className="ide-widget__tree overflow-auto">
          {files.length === 0 ? (
            <p className="p-2 text-xs text-[var(--text-secondary)]">No files</p>
          ) : (
            <ul>
              {files.map((file) => (
                <li key={file.path}>
                  <button
                    type="button"
                    className={`ide-widget__file ${file.path === state.open_file ? "ide-widget__file--active" : ""}`}
                    onClick={() => void openFile(file.path)}
                  >
                    {file.name}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </aside>
        <div className="ide-widget__editor min-h-0">
          <Editor
            height="100%"
            theme="vs-dark"
            language="plaintext"
            value={editorValue}
            onChange={(value) => setEditorValue(value ?? "")}
            options={{
              minimap: { enabled: false },
              fontSize: 12,
              wordWrap: "on",
              scrollBeyondLastLine: false,
            }}
          />
        </div>
        <div ref={terminalHostRef} className="ide-widget__terminal min-h-0" />
      </div>
    </div>
  );
}
