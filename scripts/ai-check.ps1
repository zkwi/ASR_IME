$ErrorActionPreference = "Stop"

Write-Host "== VoxType AI Local Check =="

if (-not (Test-Path "package.json")) {
  Write-Error "Please run this script from the repository root."
}

function Invoke-CheckedCommand {
  param(
    [string]$Name,
    [scriptblock]$Command
  )

  Write-Host "`n$Name"
  $global:LASTEXITCODE = 0
  & $Command
  if ($LASTEXITCODE -ne 0) {
    throw "$Name failed with exit code $LASTEXITCODE"
  }
}

Invoke-CheckedCommand "[1/7] Frontend type check" { npm run check }

Invoke-CheckedCommand "[2/7] Frontend build" { npm run build }

Invoke-CheckedCommand "[3/7] Secret scan" { npm run scan:secrets }

Invoke-CheckedCommand "[4/7] Secret scan self-test" { npm run test:secrets }

Push-Location ".\src-tauri"
try {
  Invoke-CheckedCommand "[5/7] Rust fmt check" { cargo fmt --check }

  Invoke-CheckedCommand "[6/7] Rust check" { cargo check }

  Invoke-CheckedCommand "[7/7] Rust tests" { cargo test }
} finally {
  Pop-Location
}

Write-Host "`nAll local checks passed."
