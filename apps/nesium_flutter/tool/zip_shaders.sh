#!/bin/bash
# apps/nesium_flutter/tool/zip_shaders.sh

# Cross-platform script to zip shaders.
# On Windows, it uses PowerShell + .NET for speed.
# On macOS/Linux, it uses the standard zip command.

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ASSETS_DIR="$SCRIPT_DIR/../assets"
SHADERS_DIR="$ASSETS_DIR/shaders"
ZIP_FILE="$ASSETS_DIR/shaders.zip"

echo "Zipping shaders from $SHADERS_DIR to $ZIP_FILE..."

if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    # Windows
    powershell.exe -NoProfile -Command "
        \$shaderDir = Resolve-Path '$SHADERS_DIR';
        \$zipFile = '$ZIP_FILE';
        if (Test-Path \$zipFile) { Remove-Item \$zipFile }
        Add-Type -AssemblyName 'System.IO.Compression.FileSystem';
        \$zip = [System.IO.Compression.ZipFile]::Open(\$zipFile, [System.IO.Compression.ZipArchiveMode]::Create);
        Get-ChildItem -Path \$shaderDir -Recurse | Where-Object { \$_.FullName -notmatch '\\\\.git(\$|\\\\)' } | ForEach-Object {
            if (-not \$_.PSIsContainer) {
                \$relativePath = \$_.FullName.Substring(\$shaderDir.Path.Length + 1).Replace('\\', '/');
                [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(\$zip, \$_.FullName, \$relativePath, [System.IO.Compression.CompressionLevel]::Optimal);
            }
        };
        \$zip.Dispose();
    "
else
    # macOS / Linux
    if command -v zip >/dev/null 2>&1; then
        rm -f "$ZIP_FILE"
        cd "$SHADERS_DIR" && zip -r "$ZIP_FILE" . -x "*.git*"
    else
        echo "Error: 'zip' command not found."
        exit 1
    fi
fi

echo "Done!"
