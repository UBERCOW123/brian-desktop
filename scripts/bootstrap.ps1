#Requires -Version 5.1
<#
.SYNOPSIS
  Bootstrap Brian Desktop dev environment on Windows.
#>
param(
    [switch]$SkipRust,
    [switch]$SkipNpm
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host "==> Brian Desktop bootstrap" -ForegroundColor Cyan

if (-not (Test-Path "vendor/core/docs/agent/SYNC_STRATEGY.md")) {
    Write-Host "==> Initializing vendor/core submodule"
    git submodule update --init --recursive
}

Write-Host "==> Generating placeholder icons"
python "$PSScriptRoot/generate-placeholder-icons.py"
if (Get-Command npx -ErrorAction SilentlyContinue) {
    npx tauri icon "src-tauri/icons/128x128.png" | Out-Null
}

if (-not $SkipRust) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "Rust not found. Install from https://rustup.rs/ then re-run this script." -ForegroundColor Yellow
        Write-Host "  winget install Rustlang.Rustup"
    } else {
        Write-Host "==> rustfmt / clippy / check"
        cargo fmt --all
        cargo clippy --workspace --all-targets -- -D warnings
        cargo test --workspace
        cargo check --workspace
    }
}

if (-not $SkipNpm) {
    if (-not (Test-Path "node_modules")) {
        Write-Host "==> npm install"
        npm install
    }
}

if (-not (Test-Path ".env") -and (Test-Path ".env.example")) {
    Copy-Item ".env.example" ".env"
    Write-Host "Created .env from .env.example — fill in Supabase values." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Ready. Next:" -ForegroundColor Green
Write-Host "  npm run tauri dev"
