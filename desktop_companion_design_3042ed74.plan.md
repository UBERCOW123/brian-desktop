---
name: Desktop Companion Design
overview: Brian Desktop (Rust/Tauri + Vite) shares CORE's Supabase project and local-first sync contract. Phases 0–2.5 (contracts, auth, sync, workbench grid, Brian UI, Assist) are done. Next focus is Work Room, desktop agent, and polish.
todos:
  - id: design-system-parity
    content: "Phase 2: Port CORE design tokens, glass cards, VSCode-style dockable shell, timeline + widget grid styling from mobile"
    status: completed
  - id: brian-assist-ui
    content: "Phase 2b: Brian Assist chat panel with preview/confirm cards matching agent_chat_screen.dart"
    status: completed
  - id: workbench-redesign
    content: "Phase 2.5: Full-viewport widget workbench — Brian UI kit, CSS glass, desktop shell widgets, seeded layout, add-widget drawer"
    status: completed
  - id: workroom-light
    content: "Phase 3: Work Room — WebView browser, markdown notebook, grid spreadsheet, Monaco IDE, toolbox widgets"
    status: pending
  - id: desktop-agent
    content: "Phase 4: Extend MCP with OS tools (filesystem, shell, UI, clipboard) behind preview/confirm"
    status: pending
  - id: mobile-coordination
    content: "Coordinate core PRs: theme sync via app_setting, workroom_session migration, new widget types"
    status: pending
  - id: onlyoffice-v15
    content: "Phase 5: optional OnlyOffice embed for Word/PowerPoint/Excel fidelity"
    status: pending
isProject: false
---

# CORE Desktop Companion — Remaining Plan

## Completed (Phases 0–2.5)

| Phase | Delivered |
|-------|-----------|
| **0 — Contracts & foundation** | `vendor/core` submodule (`core-contract-v0.1`), `core-contracts` parity tests, `core-db` SQLite v14 mirror, PKCE Apple auth + keyring session, device registration, read projections |
| **1 — Dashboard data** | Push/pull sync (`core-sync`), timeline + task quick-add, 8-column widget grid (drag/resize + collision normalize), marketplace install/remove |
| **2 — Design system** | 18 mobile theme presets (`scripts/generate-theme-presets.mjs`), Tremor-derived **Brian UI** kit (`src/ui/`), CSS Gaussian glass (`brian-glass`), timeline spine styling |
| **2.5 — Workbench redesign** | Full-viewport **widget workbench** (no fixed 3-column mobile shell). Desktop shell widgets (`desktop_assist`, `desktop_timeline`, `desktop_sync`) as grid tiles. Default seeded layout on empty DB. Slim titlebar + grid toolbar (lock, add widget, reset layout). Add-widget drawer with marketplace. Assist as hero center tile (`AssistWidget.tsx`). WebGL liquid-glass removed. **dockview** retained for Phase 3 Work Room splits only. |

Stack in production: **Tauri 2** + **Vite/React/TypeScript** UI + **Brian UI** (Tremor + CORE tokens) + Rust crates (`core-auth`, `core-db`, `core-sync`, `core-mcp` stubs).

### Phase 2.5 success criteria (met)

- Fresh DB → default layout shows Assist (center-large), timeline (left), sync tile, tasks, capture
- Unlock grid → drag/resize all tiles including Assist
- Lock grid → handles hidden; layout persists after restart
- Glass readable: frosted background only; content never blurred
- Add widget drawer installs catalog tiles onto grid
- No permanent side columns, sync footer, or mobile-style WIDGETS/MARKETPLACE tabs

---

## Current gap

Work Room panes, full MCP orchestrator wiring, and desktop OS agent tools remain. Assist chat uses UI-complete preview/confirm with stub responses until Phase 4.

---

## Phase 3 — Work Room (lightweight)

Multi-pane workspace inside a dock group or dedicated window:

| Pane | v1 | v1.5 |
|------|-----|------|
| Browser | Tauri WebView | Extension bridge |
| Notebook | Markdown editor + preview | Wikilinks, capture ingest |
| Spreadsheet | CSV grid, formulas-lite | OnlyOffice Calc |
| Slides | Markdown deck | OnlyOffice Impress |
| Word | Rich markdown + DOCX export | OnlyOffice Writer |
| IDE | Monaco in WebView + terminal panel | LSP, git status |
| Toolbox | Calculator, canvas, paste history | Marketplace-addable |

**State:** persist open tabs + pane sizes as `app_setting` or new `workroom_session` kind — **requires `core/supabase/migrations/` PR first**.

---

## Phase 4 — Desktop agent + connectors

Extend mobile MCP (44 in-app tools) with **OS capability tier** (always confirm for destructive ops):

| Capability | Examples |
|------------|----------|
| Filesystem | Read/write user-granted dirs; open in Work Room |
| Shell | Scoped local terminal |
| Window/UI | Focus panes, split layout, open URLs in embedded browser |
| Clipboard | Paste history widget |
| Connectors | Desktop Gmail/GitHub OAuth (mobile Google inlet is iOS-only) |

Port tool schemas from `core_mcp_registry.dart`; keep model-first + preview/confirm.

---

## Phase 5 — Polish & hybrid office

- OnlyOffice embed for heavy docs
- Theme sync option (`app_setting` — migrate mobile off SharedPreferences)
- Supabase Realtime or `companion_signal` records for cross-device hints
- Bundle import/export parity (`CoreBundleService`)

---

## Integration pitfalls (still apply)

1. **Local-first only** — no direct Supabase CRUD; all writes via SQLite + `sync_outbox`.
2. **Stable UUIDs** — never regenerate record IDs on desktop.
3. **Desktop OAuth redirect** — `com.celix.core.desktop://login-callback` (separate from mobile).
4. **`widgetType` + `configJson`** — must match `WIDGET_AGENT_METADATA_SEED.json` exactly.
5. **Grid clamp on sync** — mobile positions on 4–6 columns may overflow desktop 8-col grid without `GridLayoutEngine.fitLayoutToGrid`.
6. **New record kinds** — `workroom_session`, layout sync keys need core migration before shipping.
7. **Never ship `service_role`** in the binary.

---

## Core repo coordination (parallel PRs)

- `app_setting` keys for theme + layout sync (optional account flag)
- `workroom_session` kind migration before Work Room ships
- New desktop `widgetType` entries in catalog + seed regeneration
- Desktop OAuth redirect in Supabase Auth dashboard
- Part C2 multi-device sync before marketing "seamless sync"

---

## Success criteria (remaining)

- Work Room opens browser + notebook + IDE with agent driving panes under explicit approval (dockview splits inside a Work Room widget tile)
- Same Apple account shows synced tasks, widgets, and plans after outbox drain
- Brian Assist preview/confirm works on both platforms with live MCP orchestrator (Phase 4)
- User can run cloud ASSIST, BYOK, or local Ollama on desktop
- Token/card visual parity with mobile at the palette level (Brian UI + CORE tokens)
