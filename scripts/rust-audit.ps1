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
  cargo audit
  if ($LASTEXITCODE -ne 0) {
    throw "cargo audit failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}
