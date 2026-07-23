# OPTIONAL: retag and push the builder image to a registry you control.
# Publishing is a deliberate manual action — this is never run by CI.
# Usage: docker/builder/push.ps1 <registry-prefix>
#   e.g. docker/builder/push.ps1 registry.gitlab.com/pos9014819/rust-desk
# Requires that you have already run `docker login` yourself. This script never
# reads, stores, or logs credentials/tokens.
param(
    [Parameter(Mandatory = $true)][string]$RegistryPrefix
)
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ImageName = "enzu/rustdesk-builder"

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "docker is not installed or not on PATH."
    exit 1
}

$Prefix  = $RegistryPrefix.TrimEnd('/')
$Version = (Get-Content (Join-Path $ScriptDir "VERSION") -Raw).Trim()
$SrcTag  = "${ImageName}:${Version}"
$DestTag = "${Prefix}/${ImageName}:${Version}"

Write-Host "Retagging $SrcTag -> $DestTag"
docker tag $SrcTag $DestTag
if ($LASTEXITCODE -ne 0) { Write-Error "docker tag failed."; exit $LASTEXITCODE }
Write-Host "Pushing $DestTag (ensure you have run 'docker login' already)"
docker push $DestTag
if ($LASTEXITCODE -ne 0) { Write-Error "docker push failed."; exit $LASTEXITCODE }
Write-Host "Done."
