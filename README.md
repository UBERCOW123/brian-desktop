# Brian Desktop (CORE Desktop Companion)

Rust/Tauri desktop companion for [CORE](https://github.com/UBERCOW123/core). Consumes the **same Supabase project** as mobile; **migrations and schema live in core**, not here.

## Contract baseline

Shared sync rules, data model, widget catalog, and Supabase migrations are vendored from core:

| What | Path in submodule |
|------|-------------------|
| Sync rules | `vendor/core/docs/agent/SYNC_STRATEGY.md` |
| Record kinds / payloads | `vendor/core/docs/agent/CORE_DATA_MODEL.md` |
| Widget catalog | `vendor/core/docs/agent/WIDGET_AGENT_METADATA_SEED.json` |
| Schema (reference) | `vendor/core/supabase/migrations/` |
| Supabase env template | `vendor/core/supabase.local.json.example` |

Pin is recorded in [`CONTRACTS_VERSION`](CONTRACTS_VERSION). Bump the submodule when mobile schema or contracts change:

```bash
cd vendor/core
git fetch --tags
git checkout core-contract-v0.2   # or target commit
cd ../..
git add vendor/core CONTRACTS_VERSION
git commit -m "Bump core contracts to core-contract-v0.2"
```

Clone with contracts:

```bash
git clone --recurse-submodules https://github.com/UBERCOW123/brian-desktop.git
```

## Submodule setup

```bash
git submodule update --init --recursive
```

## Env / secrets

Copy `vendor/core/supabase.local.json.example` values into a local `.env` (gitignored):

| Variable | Notes |
|----------|-------|
| `SUPABASE_URL` | Same project as mobile |
| `SUPABASE_ANON_KEY` | Anon/publishable key only — never `service_role` |
| OpenRouter BYOK | Desktop-only user setting |

Desktop OAuth redirect (register in Supabase when wiring auth): `com.celix.core.desktop://login-callback`

## What lives where

- **core** — Flutter mobile, Supabase migrations, edge functions, agent docs
- **brian-desktop** — Tauri app, local SQLite mirror, sync client, Work Room, OS-level agent tools

Do not duplicate Supabase SQL in this repo. New record kinds require a migration in core first.

## Layout (planned)

```
brian-desktop/
├── src-tauri/          # Tauri shell
├── crates/             # sync, db, mcp (optional workspace crates)
├── vendor/core/        # git submodule → core contracts + migrations (read-only)
└── contracts/          # reserved for generated types / CI copies if needed
```
