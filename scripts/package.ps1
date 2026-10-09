param(
    [Parameter(Mandatory = $true)][ValidatePattern('^[a-zA-Z0-9_.-]+$')][string]$Target
)
$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
$work = Join-Path ([System.IO.Path]::GetTempPath()) "flx4-pilot-$([guid]::NewGuid().ToString('N'))"
Push-Location $project
try {
    $metadataText = & cargo metadata --format-version 1 --locked --filter-platform $Target
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
    $metadata = ($metadataText -join "`n") | ConvertFrom-Json
    $root = $metadata.packages | Where-Object { $_.name -eq 'flx4-pilot' }
    $name = "flx4-pilot-$($root.version)-$Target"
    $dist = Join-Path $project 'dist'
    $stage = Join-Path $work 'files'
    New-Item -ItemType Directory -Path $stage -Force | Out-Null
    $windows = $Target -match 'windows'
    $binary = if ($windows) { 'flx4-pilot.exe' } else { 'flx4-pilot' }
    Copy-Item (Join-Path $metadata.target_directory "$Target/release/$binary") $stage
    Copy-Item README.md, MAPPING.md, STATUS.md, CONTROL-GUIDE.html, CONTROL-GUIDE.pdf, LICENSE $stage
    if ($windows) { Copy-Item 'Start FLX4 Pilot.cmd' $stage }
    $notices = @('# Dependency notices', '', 'Generated from Cargo.lock and dependency source packages.', '')
    $resolved = @{}
    foreach ($node in $metadata.resolve.nodes) { $resolved[$node.id] = $true }
    foreach ($package in $metadata.packages) {
        if ($package.id -eq $root.id -or -not $resolved.ContainsKey($package.id)) { continue }
        $label = "$($package.name)-$($package.version)"
        $notices += "## $label"
        $notices += "License expression: $($package.license)"
        $notices += "Source: $($package.repository)"
        $notices += ''
        $source = Split-Path $package.manifest_path -Parent
        $files = @(Get-ChildItem -LiteralPath $source -File | Where-Object {
            $_.Name -match '^(licen[sc]e|copying|notice|copyright)([._-].*)?$'
        })
        if ($package.license_file) {
            $declared = Get-Item -LiteralPath (Join-Path $source $package.license_file)
            if ($declared.PSIsContainer) { throw "Declared license file is a directory: $label" }
            $files += $declared
        }
        $files = @($files | Sort-Object FullName -Unique)
        if ($files.Count -eq 0) { throw "No license notice found for $label; inspect dependency before distributing" }
        $destination = Join-Path $stage "third-party/$label"
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
        foreach ($file in $files) { Copy-Item -LiteralPath $file.FullName -Destination $destination -Force }
    }
    $notices | Set-Content (Join-Path $stage 'THIRD-PARTY.md') -Encoding UTF8
    if ($windows) {
        # Cargo source archives can carry pre-1980 timestamps, which ZIP cannot encode.
        Get-ChildItem $stage -Recurse | ForEach-Object {
            if ($_.LastWriteTimeUtc.Year -lt 1980 -or $_.LastWriteTimeUtc.Year -gt 2107) {
                $_.LastWriteTimeUtc = [DateTime]::SpecifyKind([DateTime]'2000-01-01', [DateTimeKind]::Utc)
            }
        }
        $archive = Join-Path $work "$name.zip"
        Compress-Archive -Path "$stage/*" -DestinationPath $archive -Force
    } else {
        $archive = Join-Path $work "$name.tar.gz"
        & tar -czf $archive -C $stage .
        if ($LASTEXITCODE -ne 0) { throw 'tar failed' }
    }
    $hash = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $(Split-Path $archive -Leaf)" | Set-Content "$archive.sha256" -Encoding ASCII
    # Replace published files only after the archive and checksum are complete.
    New-Item -ItemType Directory -Path $dist -Force | Out-Null
    Move-Item -LiteralPath $archive -Destination $dist -Force
    Move-Item -LiteralPath "$archive.sha256" -Destination $dist -Force
    Write-Host "Packaged $(Join-Path $dist (Split-Path $archive -Leaf))"
} finally {
    try {
        if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
    } finally {
        Pop-Location
    }
}
