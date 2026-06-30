export const COLUMN_COUNT = 8;
export const CELL_PX = 44;
export const GRID_GAP = 8;

export interface HealthResponse {
  ok: boolean;
  schema_version: number;
  contracts_present: boolean;
  supabase_configured: boolean;
  signed_in: boolean;
}

export interface SessionInfo {
  signed_in: boolean;
  user_id: string | null;
  email: string | null;
  workspace_id: string | null;
}

export interface TaskQueueItem {
  record_id: string;
  title: string;
  display_title: string;
  status: "active" | "completed";
  is_overdue: boolean;
}

export interface TimelineEntry {
  id: string;
  entry_type: "task" | "alert" | "note" | "milestone" | "achievement";
  title: string;
  subtitle: string | null;
  date: string;
  is_completed: boolean;
  is_critical: boolean;
}

export interface WidgetInstance {
  id: string;
  widget_type: string;
  pos_x: number;
  pos_y: number;
  width: number;
  height: number;
  config_json: string;
}

export interface WidgetCatalogEntry {
  widgetType: string;
  displayName: string;
  category: string | null;
  surfaceKind: string | null;
  allowMultiple: boolean;
  defaultSize: { width: number; height: number } | null;
}

export interface WidgetCatalog {
  schema_version: number;
  widgets: WidgetCatalogEntry[];
}

export interface WidgetLayoutInput {
  id: string;
  pos_x: number;
  pos_y: number;
  width: number;
  height: number;
}

export function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

export function formatDate(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
