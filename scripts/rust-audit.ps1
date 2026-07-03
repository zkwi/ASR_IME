$ErrorActionPreference = "Stop"

if (-not (Test-Path "package.json") -or -not (Test-Path ".\src-tauri\Cargo.lock")) {
  Write-Error "Please run this script from the repository root."
}

Write-Host "== VoxType Rust Dependency Audit =="

$global:LASTEXITCODE = 0
cargo audit --version
if ($LASTEXITCODE -ne 0) {
  throw "cargo-audit is not installed. Install it with: cargo install cargo-audit --locked"
}

Push-Location ".\src-tauri"
try {
  $global:LASTEXITCODE = 0
  # Tauri 2.11.5 still reaches quick-xml through plist 1.9.0, which currently
  # pins quick-xml below RustSec's fixed 0.41.0 line. VoxType does not parse
  # untrusted plist/XML at runtime; keep these two upstream advisories explicit
  # so any new vulnerability still fails the release check.
  cargo audit --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195
  if ($LASTEXITCODE -ne 0) {
    throw "cargo audit failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}
