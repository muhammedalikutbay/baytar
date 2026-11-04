[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$updates = @()

# Prepare and validate every replacement before writing any files. Preserve
# existing formatting and line endings instead of reserializing the documents.
foreach ($relativePath in @('Cargo.toml', 'app/package.json', 'app/src-tauri/tauri.conf.json')) {
    $path = Join-Path $repoRoot $relativePath
    $content = [System.IO.File]::ReadAllText($path)
    if ($relativePath -eq 'Cargo.toml') {
        $pattern = '(?ms)(^\[workspace\.package\]\r?\n(?:(?!^\[).)*?^version\s*=\s*")[^"]+("[^\r\n]*)'
    } else {
        $null = $content | ConvertFrom-Json
        $pattern = '(?m)(^  "version"\s*:\s*")[^"]+("\s*,?[^\r\n]*)'
    }
    $regex = [regex]$pattern
    if ($regex.Matches($content).Count -ne 1) {
        throw "Expected exactly one version field in $relativePath; no files were changed."
    }
    $updated = $regex.Replace($content, [System.Text.RegularExpressions.MatchEvaluator]{
        param($match)
        $match.Groups[1].Value + $Version + $match.Groups[2].Value
    })
    $updates += @{ Path = $path; Content = $updated }
}

foreach ($update in $updates) {
    [System.IO.File]::WriteAllText($update.Path, $update.Content, $utf8)
}
Write-Output "Updated workspace, frontend, and Tauri versions to $Version."
