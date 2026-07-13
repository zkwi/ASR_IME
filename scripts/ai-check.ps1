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

Invoke-CheckedCommand "[1/11] Frontend type check" { npm run check }

Invoke-CheckedCommand "[2/11] Frontend build" { npm run build }

Invoke-CheckedCommand "[3/11] Frontend unit tests" { npm run test:unit }

Invoke-CheckedCommand "[4/11] Secret scan" { npm run scan:secrets }

Invoke-CheckedCommand "[5/11] Secret scan self-test" { npm run test:secrets }

Invoke-CheckedCommand "[6/11] Governance checks" { npm run check:governance }

Invoke-CheckedCommand "[7/11] Governance check self-test" { npm run test:governance }

Invoke-CheckedCommand "[8/11] Release preflight self-test" { npm run test:release-preflight }

Push-Location ".\src-tauri"
try {
  Invoke-CheckedCommand "[9/11] Rust fmt check" { cargo fmt --check }

  Invoke-CheckedCommand "[10/11] Rust check" { cargo check }

  Invoke-CheckedCommand "[11/11] Rust tests" { cargo test }
} finally {
  Pop-Location
}

Write-Host "`nAll local checks passed."
