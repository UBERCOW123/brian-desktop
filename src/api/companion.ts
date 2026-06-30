import { invoke } from "@tauri-apps/api/core";

export interface BrowserState {
  url: string;
  history: string[];
  history_index: number;
}

export interface NotebookNote {
  id: string;
  title: string;
  body: string;
  updated_at: string;
}

export interface IdeState {
  root_path: string | null;
  open_file: string | null;
}

export interface IdeFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export async function createTaskFromAssist(title: string): Promise<string> {
  return invoke<string>("create_task", { title });
}

export async function getBrowserState(): Promise<BrowserState> {
  return invoke<BrowserState>("get_browser_state");
}

export async function setBrowserState(state: BrowserState): Promise<void> {
  return invoke("set_browser_state", { payload: state });
}

export async function listNotes(): Promise<NotebookNote[]> {
  return invoke<NotebookNote[]>("list_notes");
}

export async function createNote(title: string, body: string): Promise<string> {
  return invoke<string>("create_note", { title, body });
}

export async function updateNote(id: string, title: string, body: string): Promise<void> {
  return invoke("update_note", { id, title, body });
}

export async function getIdeState(): Promise<IdeState> {
  return invoke<IdeState>("get_ide_state");
}

export async function setIdeState(state: IdeState): Promise<void> {
  return invoke("set_ide_state", { payload: state });
}

export async function pickIdeFolder(): Promise<string | null> {
  return invoke<string | null>("pick_ide_folder");
}

export async function listIdeFiles(): Promise<IdeFileEntry[]> {
  return invoke<IdeFileEntry[]>("list_ide_files");
}

export async function readIdeFile(path: string): Promise<string> {
  return invoke<string>("read_ide_file", { path });
}

export async function writeIdeFile(path: string, contents: string): Promise<void> {
  return invoke("write_ide_file", { path, contents });
}

export function refreshWorkbench(): void {
  window.dispatchEvent(new CustomEvent("core:refresh"));
}
