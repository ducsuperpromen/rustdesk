# Build the ENZU Android builder image with the pinned tag from .\VERSION.
# Does NOT publish. Run from anywhere — paths are resolved relative to the repo root.
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = (Resolve-Path (Join-Path $ScriptDir "..\..")).Path
$ImageName = "enzu/rustdesk-builder"

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "docker is not installed or not on PATH."
    exit 1
}

$Version = (Get-Content (Join-Path $ScriptDir "VERSION") -Raw).Trim()
if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Error "docker/builder/VERSION is empty."
    exit 1
}

$Tag = "${ImageName}:${Version}"
Write-Host "Building image: $Tag"
Write-Host "  Dockerfile : docker/builder/Dockerfile"
Write-Host "  context    : $RepoRoot"

docker build -f (Join-Path $ScriptDir "Dockerfile") -t $Tag $RepoRoot
if ($LASTEXITCODE -ne 0) { Write-Error "docker build failed."; exit $LASTEXITCODE }

Write-Host "Done. Built $Tag (not published)."
