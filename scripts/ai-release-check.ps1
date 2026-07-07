$ErrorActionPreference = "Stop"

Write-Host "== VoxType AI Release Check =="

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

Invoke-CheckedCommand "[1/5] Local checks" { .\scripts\ai-check.ps1 }

Invoke-CheckedCommand "[2/5] NPM dependency audit" { npm run audit:npm }

Invoke-CheckedCommand "[3/5] Rust dependency audit" { npm run audit:rust }

Push-Location ".\src-tauri"
try {
  Invoke-CheckedCommand "[4/5] Rust clippy" { cargo clippy --all-targets -- -D warnings }
} finally {
  Pop-Location
}

Invoke-CheckedCommand "[5/5] Tauri debug build" { npx tauri build --debug --no-bundle }

Write-Host "`nRelease checks passed."
