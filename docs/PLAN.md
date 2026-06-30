# Implementation plan

Scaffold is complete. Feature work follows the [Desktop Companion Design](https://github.com/UBERCOW123/brian-desktop) plan (Cursor: `desktop_companion_design_3042ed74`).

## Phase checklist

| Phase | Scope | Crates / paths |
|-------|--------|----------------|
| **0 — Contracts & foundation** | Contract tests, PKCE auth, device registration, SQLite projections | `core-contracts`, `core-db`, `core-sync`, `src-tauri` |
| **1 — Dashboard** | Timeline, widget grid, marketplace, Brian Assist, push/pull sync | UI `src/`, `core-sync` |
| **2 — Work Room** | Browser, notebook, spreadsheet, Monaco, layout persistence | `src-tauri` WebViews, new `app_setting` keys in core |
| **3 — Desktop agent** | OS tools, Gmail/GitHub OAuth, clipboard widgets | `core-mcp`, `src-tauri` |
| **1.5 — Office** | OnlyOffice embed (optional) | Work Room panes |

## Start Phase 0

Recommended order:

1. **Contract parity tests** — assert Rust record kinds / payload keys against `vendor/core/docs/agent/CORE_DATA_MODEL.md` and widget seed entries.
2. **Auth** — PKCE + keyring session; register `com.celix.core.desktop://login-callback` in Supabase.
3. **Device registration** — mirror mobile `DeviceRegistrationService` (`platform = windows`).
4. **Sync engine** — implement `SyncPushClient` / `SyncPullClient` per `SYNC_STRATEGY.md`.
5. **Read projections** — tasks, timeline, widgets from `core_records`.

## Verify scaffold

```powershell
.\scripts\bootstrap.ps1
cargo test --workspace
npm run tauri dev
```

## Core repo coordination (parallel PRs)

- Push `core-contract-v0.1` tag from `core`
- Desktop OAuth redirect in Supabase Auth
- `app_setting` theme sync migration (optional)
- `workroom_session` kind migration before Work Room ships
