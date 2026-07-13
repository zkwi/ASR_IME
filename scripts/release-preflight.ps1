function Assert-ReleaseBuildArtifactWritable {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path
  )

  if (-not (Test-Path -LiteralPath $Path)) {
    return
  }

  $fullPath = [System.IO.Path]::GetFullPath($Path)
  try {
    # Reproduce Tauri's exclusive overwrite requirement before expensive release checks.
    $stream = [System.IO.File]::Open(
      $fullPath,
      [System.IO.FileMode]::Open,
      [System.IO.FileAccess]::ReadWrite,
      [System.IO.FileShare]::None
    )
    $stream.Dispose()
  } catch {
    throw "Release build artifact is locked: $fullPath. Please close the running VoxType debug app and retry."
  }
}
