[CmdletBinding()]
param(
    [string]$OutputDirectory = (Join-Path (Get-Location) 'out\kreuzen-windows-x64')
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$output = [System.IO.Path]::GetFullPath($OutputDirectory)

Push-Location $repo
try {
    cargo build --locked --release -p kreuzen-cli
    $exe = Join-Path $repo 'target\release\kreuzen.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
        throw "Release executable was not produced: $exe"
    }

    New-Item -ItemType Directory -Force -Path $output | Out-Null
    Copy-Item -LiteralPath $exe -Destination (Join-Path $output 'kreuzen.exe') -Force
    Copy-Item -LiteralPath (Join-Path $repo 'README.md') -Destination $output -Force
    Copy-Item -LiteralPath (Join-Path $repo 'README_CN.md') -Destination $output -Force
    Copy-Item -LiteralPath (Join-Path $repo 'docs\p2-validation.zh-CN.md') -Destination $output -Force
    Copy-Item -LiteralPath (Join-Path $repo 'docs\p4-sky-fc-validation.zh-CN.md') -Destination $output -Force

    $artifact = Join-Path $output 'kreuzen.exe'
    [pscustomobject]@{
        Artifact = $artifact
        Size = (Get-Item -LiteralPath $artifact).Length
        Sha256 = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash
    }
} finally {
    Pop-Location
}
