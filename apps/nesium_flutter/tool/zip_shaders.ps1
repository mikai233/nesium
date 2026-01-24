# apps/nesium_flutter/tool/zip_shaders.ps1
$scriptDir = Split-Path $MyInvocation.MyCommand.Path -Parent
$shaderDir = [System.IO.Path]::GetFullPath((Join-Path $scriptDir "../assets/shaders"))
$zipFile = [System.IO.Path]::GetFullPath((Join-Path $scriptDir "../assets/shaders.zip"))

if (Test-Path $zipFile) {
    Remove-Item $zipFile
}

Write-Host "Zipping shaders from $shaderDir to $zipFile..."

Add-Type -AssemblyName "System.IO.Compression.FileSystem"
$compressionLevel = [System.IO.Compression.CompressionLevel]::Optimal
$zip = [System.IO.Compression.ZipFile]::Open($zipFile, [System.IO.Compression.ZipArchiveMode]::Create)

try {
    Get-ChildItem -Path $shaderDir -Recurse | Where-Object { $_.FullName -notmatch "\\\.git($|\\)" } | ForEach-Object {
        if (-not $_.PSIsContainer) {
            $relativePath = $_.FullName.Substring($shaderDir.Length + 1).Replace('\', '/')
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $_.FullName, $relativePath, $compressionLevel) | Out-Null
        }
    }
}
finally {
    $zip.Dispose()
}

Write-Host "Done! File size: $(([System.IO.File]::ReadAllBytes($zipFile).Length / 1MB).ToString('F2')) MB"
