# Development environment

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable (see `rust-toolchain.toml`) | [rustup.rs](https://rustup.rs/) |
| Node.js | 20+ | Vite + Tauri CLI |
| MSVC Build Tools | latest | Windows only — required for `rusqlite`/`tauri` |

## First-time setup

```powershell
cd c:\Users\georg\Documents\github\brian-desktop
git clone --recurse-submodules https://github.com/UBERCOW123/brian-desktop.git .   # if fresh
.\scripts\bootstrap.ps1
```

Bootstrap will:

1. Init `vendor/core` submodule
2. Generate placeholder app icons
3. `cargo fmt`, `clippy`, `test`, `check` (when Rust is installed)
4. `npm install`
5. Copy `.env.example` → `.env`

## Environment

Edit `.env` with the **same Supabase project** as CORE mobile:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
```

Never put `service_role` in the desktop client.

## Daily dev

```powershell
npm run tauri dev
```

Frontend: Vite on `http://localhost:1420`  
Backend: Tauri watches workspace crates (`core-db`, `core-sync`, …).

## Useful commands

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run build
npm run tauri build
```

Replace placeholder icons with a real asset:

```powershell
# place a 1024x1024 PNG at assets/app-icon.png, then:
npm run tauri icon assets/app-icon.png
```

## Contract bumps

When mobile changes schema or agent docs:

```powershell
cd vendor/core
git fetch --tags
git checkout <tag-or-commit>
cd ../..
# update CONTRACTS_VERSION
git add vendor/core CONTRACTS_VERSION
git commit -m "Bump core contracts"
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `vendor/core` empty | `git submodule update --init --recursive` |
| `contracts missing` at runtime | Same as above |
| Rust link errors on Windows | Install “Desktop development with C++” in Visual Studio Build Tools |
| Submodule tag not found | Push tag from `core`: `git push origin core-contract-v0.1` |
