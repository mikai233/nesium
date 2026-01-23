# PowerShell Script to test Nesium Inno Setup Installer locally

$ErrorActionPreference = 'Stop'

# 1. Define Paths (Relative to script location)
$WindowsDir = $PSScriptRoot
$FlutterAppDir = Split-Path $WindowsDir -Parent
$ProjectDir = Split-Path $FlutterAppDir -Parent
$IssScript = Join-Path $WindowsDir "installer.iss"

Write-Host "--- Nesium Installer Test Script ---" -ForegroundColor Cyan

# 2. Check for Inno Setup
$iscc = Get-Command iscc -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
if (-not $iscc) {
    $standardPaths = @(
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe"
    )
    foreach ($path in $standardPaths) {
        if (Test-Path $path) {
            $iscc = $path
            break
        }
    }
}

if (-not $iscc) {
    Write-Error "Inno Setup Compiler (ISCC.exe) not found. Please install it from https://jrsoftware.org/isdl.php"
}
Write-Host "Using ISCC: $iscc"

# 3. Build Flutter Windows (Optional but recommended)
$buildChoice = Read-Host "Do you want to rebuild the Flutter Windows project? (y/n)"
if ($buildChoice -eq 'y') {
    Write-Host "Building Flutter Windows release..." -ForegroundColor Yellow
    Push-Location $FlutterAppDir
    flutter build windows --release
    Pop-Location
}

# 4. Determine Build Source
$src = Join-Path $FlutterAppDir "build\windows\x64\runner\Release"
if (-not (Test-Path $src)) {
    $src = Join-Path $FlutterAppDir "build\windows\runner\Release"
}
if (-not (Test-Path $src)) {
    Write-Error "Flutter build output not found at $src. Please build the project first."
}

# 5. Extract Version from pubspec.yaml
Write-Host "Syncing version from pubspec.yaml..." -ForegroundColor Yellow
$pubspecPath = Join-Path $FlutterAppDir "pubspec.yaml"
$pubspecContent = Get-Content $pubspecPath -Raw
if ($pubspecContent -match 'version:\s*([^\s+]+)') {
    $version = $Matches[1]
    Write-Host "Detected Version: $version" -ForegroundColor Green
} else {
    Write-Host "Warning: Could not detect version from pubspec.yaml, falling back to 1.0.0-local" -ForegroundColor Gray
    $version = "1.0.0-local"
}

# 6. Compile Installer
Write-Host "Compiling Installer..." -ForegroundColor Yellow
$outputDir = Join-Path $FlutterAppDir "build\installer"
if (-not (Test-Path $outputDir)) { New-Item -ItemType Directory $outputDir | Out-Null }

& $iscc /DMyAppVersion="$version" /DSourceDir="$src" /O"$outputDir" "$IssScript"

Write-Host "--- Done! ---" -ForegroundColor Green
Write-Host "Your installer is ready at: $outputDir"
explorer $outputDir
