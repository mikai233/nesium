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
    # Windows: Use the dedicated PowerShell script for consistency and performance
    # Passing the full path to ensure it finds the script relative to this script's directory
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$SCRIPT_DIR/zip_shaders.ps1")"
else
    # macOS / Linux
    if command -v zip >/dev/null 2>&1; then
        rm -f "$ZIP_FILE"
        EXCLUDES=(
            "*.git*"
            "*/bezel/*"
            "*/handheld/*"
            "*/stereoscopic-3d/*"
            "*/deinterlacing/*"
            "*/motion-interpolation/*"
            "*/test/*"
            "*/test-patterns/*"
        )
        cd "$SHADERS_DIR" && zip -r "$ZIP_FILE" . -x "${EXCLUDES[@]}"
    else
        echo "Error: 'zip' command not found."
        exit 1
    fi
fi

echo "Done!"
