; # PowerShell Script to build Nesium (egui) Inno Setup Installer locally

$ErrorActionPreference = 'Stop'

# 1. Define Paths (Relative to script location)
$WindowsDir = $PSScriptRoot
$EguiAppDir = Split-Path $WindowsDir -Parent
$AppsDir = Split-Path $EguiAppDir -Parent
$ProjectDir = Split-Path $AppsDir -Parent
$IssScript = Join-Path $WindowsDir "installer.iss"

Write-Host "--- Nesium (egui) Installer Build Script ---" -ForegroundColor Cyan

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

# 3. Build Rust project (Optional but recommended)
$buildChoice = Read-Host "Do you want to rebuild the egui project? (y/n)"
if ($buildChoice -eq 'y') {
    Write-Host "Building nesium-egui release..." -ForegroundColor Yellow
    Push-Location $ProjectDir
    cargo build -p nesium-egui --profile release-dist
    Pop-Location
}

# 3.5 Generate Icon
Write-Host "Generating Icon..." -ForegroundColor Yellow
$GeneratedDir = Join-Path $ProjectDir "target\generated"
if (-not (Test-Path $GeneratedDir)) { New-Item -ItemType Directory $GeneratedDir -Force | Out-Null }
$IconPath = Join-Path $GeneratedDir "app_icon.ico"

Push-Location $ProjectDir
cargo run -p nesium-icon -- --out "$IconPath"
Pop-Location

# 4. Determine Build Source
$src = Join-Path $ProjectDir "target\release-dist"
if (-not (Test-Path $src)) {
    $src = Join-Path $ProjectDir "target\x86_64-pc-windows-msvc\release-dist"
}
if (-not (Test-Path $src)) {
    $src = Join-Path $ProjectDir "target\release"
}
if (-not (Test-Path $src)) {
    $src = Join-Path $ProjectDir "target\x86_64-pc-windows-msvc\release"
}

if (-not (Test-Path $src)) {
    Write-Error "Build output directory not found at $src. Please build the project first."
}

# 5. Extract Version from Cargo.toml
Write-Host "Syncing version from Cargo.toml..." -ForegroundColor Yellow
$cargoPath = Join-Path $EguiAppDir "Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw
if ($cargoContent -match 'version\s*=\s*"([^"]+)"') {
    $version = $Matches[1]
    Write-Host "Detected Version: $version" -ForegroundColor Green
} else {
    Write-Host "Warning: Could not detect version from Cargo.toml, falling back to 0.1.0-local" -ForegroundColor Gray
    $version = "0.1.0-local"
}

# 6. Compile Installer
Write-Host "Compiling Installer..." -ForegroundColor Yellow
$outputDir = Join-Path $ProjectDir "target\dist"
if (-not (Test-Path $outputDir)) { New-Item -ItemType Directory $outputDir -Force | Out-Null }

& $iscc /DMyAppVersion="$version" /DSourceDir="$src" /DMyAppIcon="$IconPath" /O"$outputDir" "$IssScript"

Write-Host "--- Done! ---" -ForegroundColor Green
Write-Host "Your installer is ready at: $outputDir"
explorer $outputDir
