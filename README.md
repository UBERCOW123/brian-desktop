# Brian Desktop (CORE Desktop Companion)

Rust/Tauri desktop companion for [CORE](https://github.com/UBERCOW123/core). Consumes the **same Supabase project** as mobile; **migrations and schema live in core**, not here.

## Quick start

```powershell
git clone --recurse-submodules https://github.com/UBERCOW123/brian-desktop.git
cd brian-desktop
.\scripts\bootstrap.ps1
npm run tauri dev
```

See [docs/SETUP.md](docs/SETUP.md) for prerequisites and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for crate layout.

**Agents:** read [`AGENTS.md`](AGENTS.md) first — explains the two-repo setup and what *not* to pull from `vendor/core`.

## Layout

```
brian-desktop/
├── src/                    # Vite + TypeScript UI
├── src-tauri/              # Tauri 2 shell
├── crates/
│   ├── core-contracts/     # vendor/core doc paths + widget seed loader
│   ├── core-db/            # SQLite schema mirror (mobile v14)
│   ├── core-sync/          # push/pull traits (SYNC_STRATEGY.md)
│   └── core-mcp/           # ASSIST + OS tool stubs
├── vendor/core/            # git submodule → core (read-only)
├── contracts/              # reserved for generated types
└── docs/
```

## Contract baseline

| What | Path in submodule |
|------|-------------------|
| Sync rules | `vendor/core/docs/agent/SYNC_STRATEGY.md` |
| Record kinds | `vendor/core/docs/agent/CORE_DATA_MODEL.md` |
| Widget catalog | `vendor/core/docs/agent/WIDGET_AGENT_METADATA_SEED.json` |
| Schema (reference) | `vendor/core/supabase/migrations/` |

Pin: [`CONTRACTS_VERSION`](CONTRACTS_VERSION). Bump submodule when mobile changes schema.

## Env / secrets

Copy `.env.example` → `.env` (same Supabase project as mobile). Never commit API keys or `service_role`.

Desktop OAuth redirect (when wired): `com.celix.core.desktop://login-callback`

## Status

**Scaffold complete** — workspace compiles, SQLite opens, contracts resolve. **Phases 0–2 shipped:** sync, dockable dashboard shell, mobile theme presets, Brian Assist UI chrome.

## What not to do

- Don't add Cargo.toml to `core` or fork migrations into desktop
- Don't ship desktop-only `core_records` kinds without a core migration first
