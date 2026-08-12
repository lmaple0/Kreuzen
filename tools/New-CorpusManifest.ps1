param(
	[Parameter(Mandatory = $true)]
	[string] $Game,

	[Parameter(Mandatory = $true)]
	[string] $Root,

	[string[]] $Include = @('*.bin', '*._sn'),

	[Parameter(Mandatory = $true)]
	[string] $Output
)

$ErrorActionPreference = 'Stop'
$resolvedRoot = (Resolve-Path -LiteralPath $Root).Path
$resolvedOutput = [System.IO.Path]::GetFullPath($Output)

$files = foreach ($pattern in $Include) {
	Get-ChildItem -LiteralPath $resolvedRoot -File -Recurse -Filter $pattern
}

$hashErrors = [System.Collections.Generic.List[object]]::new()
$entries = $files |
	Sort-Object FullName -Unique |
	ForEach-Object {
		$relativePath = [System.IO.Path]::GetRelativePath($resolvedRoot, $_.FullName).Replace('\', '/')
		try {
			[ordered]@{
				path = $relativePath
				size = $_.Length
				sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
			}
		} catch {
			$hashErrors.Add([ordered]@{
				path = $relativePath
				error = $_.Exception.Message
			})
		}
	}

$manifest = [ordered]@{
	schema = 1
	game = $Game
	root_hint = Split-Path -Leaf $resolvedRoot
	generated_utc = [DateTime]::UtcNow.ToString('o')
	files = @($entries)
	errors = @($hashErrors)
}

$parent = Split-Path -Parent $resolvedOutput
if ($parent -and -not (Test-Path -LiteralPath $parent)) {
	New-Item -ItemType Directory -Path $parent | Out-Null
}
$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $resolvedOutput -Encoding utf8NoBOM
Write-Output "Wrote $($manifest.files.Count) entries to $resolvedOutput"
if ($manifest.errors.Count -ne 0) {
	Write-Warning "$($manifest.errors.Count) files could not be hashed; see the manifest errors collection"
}
