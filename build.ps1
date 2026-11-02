$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    python scripts/package.py @args
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}
