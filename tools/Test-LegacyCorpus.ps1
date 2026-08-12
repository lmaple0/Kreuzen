[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Kreuzen,

    [Parameter(Mandatory = $true)]
    [string]$InputDirectory,

    [Parameter(Mandatory = $true)]
    [ValidateSet(
        'sky-fc', 'sky-fc-evo', 'sky-fc-kai',
        'sky-sc', 'sky-sc-evo', 'sky-sc-kai',
        'sky-3rd', 'sky-3rd-evo', 'sky-3rd-kai',
        'zero', 'zero-evo', 'zero-kai',
        'azure', 'azure-evo', 'azure-kai'
    )]
    [string]$Game,

    [ValidateSet('sjis', 'gbk')]
    [string]$Encoding = 'sjis',

    [string]$Charmap,

    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$exe = (Resolve-Path -LiteralPath $Kreuzen).Path
$inputRoot = (Resolve-Path -LiteralPath $InputDirectory).Path

if (-not $OutputDirectory) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $OutputDirectory = Join-Path (Get-Location) "legacy-corpus-$Game-$stamp"
}
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $outputRoot) {
    if (Get-ChildItem -LiteralPath $outputRoot -Force | Select-Object -First 1) {
        throw "Output directory is not empty: $outputRoot"
    }
} else {
    New-Item -ItemType Directory -Path $outputRoot | Out-Null
}

$sourceRoot = Join-Path $outputRoot 'source'
$roundtripRoot = Join-Path $outputRoot 'roundtrip'
New-Item -ItemType Directory -Force -Path $sourceRoot, $roundtripRoot | Out-Null

function Format-ReportMessage([string]$Message) {
    if ($Message.Length -le 4000) {
        return $Message
    }
    return $Message.Substring(0, 4000) + "`n[truncated]"
}

$binaryExtension = if ($Game -in @('sky-fc', 'sky-sc', 'sky-3rd')) { '._sn' } else { '.bin' }
$files = Get-ChildItem -LiteralPath $inputRoot -Recurse -File | Where-Object {
    $_.Name.EndsWith($binaryExtension, [System.StringComparison]::OrdinalIgnoreCase)
}

$rows = foreach ($file in $files) {
    $relative = [System.IO.Path]::GetRelativePath($inputRoot, $file.FullName)
    $relativeClm = [System.IO.Path]::ChangeExtension($relative, '.clm')
    $clm = Join-Path $sourceRoot $relativeClm
    $roundtrip = Join-Path $roundtripRoot $relative
    New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($clm)) | Out-Null
    New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($roundtrip)) | Out-Null

    $common = @('--game', $Game, '--enc', $Encoding)
    if ($Charmap) {
        $common += @('--charmap', (Resolve-Path -LiteralPath $Charmap).Path)
    }

    $decompileLog = (& $exe @common --output $clm $file.FullName 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        [pscustomobject]@{
            File = $relative
            Result = 'decompile_error'
            OriginalSha256 = $null
            RoundtripSha256 = $null
            Message = Format-ReportMessage $decompileLog
        }
        continue
    }

    $compileLog = (& $exe @common --output $roundtrip $clm 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        [pscustomobject]@{
            File = $relative
            Result = 'compile_error'
            OriginalSha256 = $null
            RoundtripSha256 = $null
            Message = Format-ReportMessage $compileLog
        }
        continue
    }

    $originalHash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    $roundtripHash = (Get-FileHash -LiteralPath $roundtrip -Algorithm SHA256).Hash
    [pscustomobject]@{
        File = $relative
        Result = if ($originalHash -eq $roundtripHash) { 'exact' } else { 'different' }
        OriginalSha256 = $originalHash
        RoundtripSha256 = $roundtripHash
        Message = Format-ReportMessage $compileLog
    }
}

$summary = [ordered]@{
    Game = $Game
    Encoding = $Encoding
    InputDirectory = $inputRoot
    OutputDirectory = $outputRoot
    Total = @($rows).Count
    Exact = @($rows | Where-Object Result -eq 'exact').Count
    Different = @($rows | Where-Object Result -eq 'different').Count
    DecompileErrors = @($rows | Where-Object Result -eq 'decompile_error').Count
    CompileErrors = @($rows | Where-Object Result -eq 'compile_error').Count
    GeneratedAt = (Get-Date).ToString('o')
}

$rows | Export-Csv -LiteralPath (Join-Path $outputRoot 'files.csv') -NoTypeInformation -Encoding utf8
$report = [ordered]@{ Summary = $summary; Files = @($rows) }
$report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $outputRoot 'report.json') -Encoding utf8
$summary | ConvertTo-Json
