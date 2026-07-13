$ErrorActionPreference = "Stop"

. "$PSScriptRoot\release-preflight.ps1"

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("voxtype-release-preflight-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tempDir | Out-Null

try {
  $missingPath = Join-Path $tempDir "missing.exe"
  Assert-ReleaseBuildArtifactWritable -Path $missingPath

  $writablePath = Join-Path $tempDir "writable.exe"
  [System.IO.File]::WriteAllText($writablePath, "test")
  Assert-ReleaseBuildArtifactWritable -Path $writablePath

  $lockedPath = Join-Path $tempDir "locked.exe"
  [System.IO.File]::WriteAllText($lockedPath, "test")
  $lock = [System.IO.File]::Open(
    $lockedPath,
    [System.IO.FileMode]::Open,
    [System.IO.FileAccess]::ReadWrite,
    [System.IO.FileShare]::None
  )
  try {
    $message = $null
    try {
      Assert-ReleaseBuildArtifactWritable -Path $lockedPath
    } catch {
      $message = $_.Exception.Message
    }
    if (-not $message) {
      throw "Expected a locked build artifact to fail the release preflight."
    }
    if ($message -notmatch "close.*VoxType.*debug" -or $message -notmatch [regex]::Escape($lockedPath)) {
      throw "Locked-artifact error is not actionable: $message"
    }
  } finally {
    $lock.Dispose()
  }
} finally {
  Remove-Item -LiteralPath $tempDir -Recurse -Force
}

Write-Host "[test-release-preflight] all checks passed"
