# Agents — read this first (Brian Desktop)

You are in **`brian-desktop`**: the Rust/Tauri **desktop companion** for CORE.  
This is **not** the mobile app repo.

## Two-repo layout

| Repo | Path (local) | Stack | Owns |
|------|----------------|-------|------|
| **core** | `vendor/core/` (submodule) | Flutter / Dart | Mobile app, Supabase **migrations**, edge functions, schema source of truth |
| **brian-desktop** | *this repo* | Rust / Tauri / Vite | Desktop app, local SQLite mirror, sync client port, Work Room, OS agent tools |

Same Supabase **project** in production. Different codebases and release cadence.

## Do not get lost in `vendor/core`

`vendor/core` is a **read-only git submodule** for contracts and schema reference. Agents working here should **not** treat mobile docs as the primary implementation guide.

| Use this (in brian-desktop) | For |
|-------------------------------|-----|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Crate layout, data flow |
| [`docs/PLAN.md`](docs/PLAN.md) | Implementation phases |
| [`docs/SETUP.md`](docs/SETUP.md) | Toolchain, env, submodule bumps |
| [`README.md`](README.md) | Quick orientation |

### Contract files (read selectively from submodule)

Only open these when you need sync/data/widget **contracts** — not the full mobile codebase:

- `vendor/core/docs/agent/SYNC_STRATEGY.md`
- `vendor/core/docs/agent/CORE_DATA_MODEL.md`
- `vendor/core/docs/agent/WIDGET_AGENT_METADATA_SEED.json`
- `vendor/core/supabase/migrations/` (reference only — **do not copy or fork**)

Pin: [`CONTRACTS_VERSION`](CONTRACTS_VERSION).

### Ignore for desktop implementation

Do **not** start from these mobile-centric entry points unless explicitly debugging mobile parity:

- `vendor/core/docs/agent/AGENT_CONTEXT.md` — Flutter/MCP “brainstorm bible”; mobile paths, Codemagic, widget screens
- `vendor/core/lib/**` — Dart source; reference for porting behavior, not for editing from this repo
- `vendor/core/.cursor/rules/**` — mobile Cursor rules

If mobile code must change (new record kind, OAuth redirect, migration), that work belongs in a **`core` PR** — coordinate separately; do not patch `vendor/core` in place except submodule version bumps.

## Hard rules

1. **No Supabase SQL in brian-desktop** — migrations live in `core/supabase/migrations/`.
2. **Local-first writes** — SQLite + `sync_outbox`; no direct Supabase CRUD for product state.
3. **No new `core_records` kinds** in desktop-only code without a core migration first.
4. **Never ship `service_role`** in the desktop binary.
5. **Bump submodule** when contracts change — update `vendor/core` + `CONTRACTS_VERSION`; don’t hand-edit copies in `contracts/`.

## Where to implement

| Concern | Location |
|---------|----------|
| SQLite schema | `crates/core-db/` |
| Sync push/pull | `crates/core-sync/` |
| ASSIST / MCP | `crates/core-mcp/`, `src-tauri/` |
| UI shell | `src/`, `src-tauri/` |
| Contract loaders / tests | `crates/core-contracts/` |

## Submodule maintenance

```bash
git submodule update --init --recursive
```

Clone with submodules: `git clone --recurse-submodules …`

## Related repo

- Mobile: https://github.com/UBERCOW123/core
- Desktop: https://github.com/UBERCOW123/brian-desktop (this repo)
