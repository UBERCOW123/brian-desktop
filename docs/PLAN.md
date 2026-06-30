# Implementation plan

Phases 0–2 are complete. Remaining work follows [`desktop_companion_design_3042ed74.plan.md`](../desktop_companion_design_3042ed74.plan.md).

## Phase checklist

| Phase | Scope | Status |
|-------|--------|--------|
| **0 — Contracts & foundation** | Contract tests, PKCE auth, device registration, SQLite projections | Done |
| **1 — Dashboard data** | Timeline, widget grid, marketplace, push/pull sync | Done |
| **2 — Design system & shell** | Theme presets, glass cards, dockview shell, Assist UI chrome | Done |
| **3 — Work Room** | Browser, notebook, spreadsheet, Monaco, layout persistence | Next |
| **4 — Desktop agent** | OS tools, Gmail/GitHub OAuth, MCP orchestrator | Planned |
| **5 — Office** | OnlyOffice embed (optional) | Planned |

## Theme regeneration

When mobile `theme_presets.dart` changes:

```powershell
npm run generate:theme
```

## Verify

```powershell
.\scripts\bootstrap.ps1
cargo test --workspace
npm run build
npm run tauri dev
```
