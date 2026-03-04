param(
    [string]$RomRoot = "F:\nesrom",
    [string]$KageRomPath = "",
    [string]$GimmickRomPath = "",
    [string]$NekketsuRomPath = "",
    [int]$StartFrame = 0,
    [int]$EndFrame = 3600,
    [double]$MaxRmse = 0.0,
    [string]$MesenDllPath = "F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\Mesen.dll",
    [string]$OutDir = "F:\CLionProjects\nesium\target\compare\audio_align_py_regression"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

if (-not (Test-Path $MesenDllPath)) {
    throw "Mesen.dll not found: $MesenDllPath"
}

$KageRomPath = if ($KageRomPath) { $KageRomPath } else { [string]$env:NESIUM_ROM_KAGE }
$GimmickRomPath = if ($GimmickRomPath) { $GimmickRomPath } else { [string]$env:NESIUM_ROM_GIMMICK }
$NekketsuRomPath = if ($NekketsuRomPath) { $NekketsuRomPath } else { [string]$env:NESIUM_ROM_NEKKETSU }

$cases = @(
    @{
        Name = "kage"
        explicit = $KageRomPath
        candidates = @("kage*.nes", "shadow*.nes")
    },
    @{
        Name = "gimmick"
        explicit = $GimmickRomPath
        candidates = @("gimmick*.nes")
    },
    @{
        Name = "nekketsu"
        explicit = $NekketsuRomPath
        candidates = @("nekketsu*.nes")
    }
)

function Resolve-RomPath {
    param(
        [string]$Explicit,
        [string[]]$Candidates
    )

    if (-not [string]::IsNullOrWhiteSpace($Explicit)) {
        if (-not (Test-Path $Explicit)) {
            throw "explicit ROM path does not exist: $Explicit"
        }
        return (Resolve-Path $Explicit).Path
    }

    foreach ($pattern in $Candidates) {
        $match = Get-ChildItem -Path $RomRoot -File -Filter $pattern -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($null -ne $match) {
            return $match.FullName
        }
    }

    return $null
}

function Parse-Rmse {
    param([string[]]$Lines)

    for ($i = $Lines.Count - 1; $i -ge 0; $i--) {
        $line = $Lines[$i]
        if ($line -match "Full RMSE:\s+([0-9.]+)") {
            return [double]$matches[1]
        }
    }

    throw "unable to parse 'Full RMSE' from analyzer output"
}

function Quote-CmdArg {
    param([string]$Value)
    if ($null -eq $Value) {
        return '""'
    }
    return '"' + ($Value -replace '"', '\"') + '"'
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$logDir = Join-Path $OutDir "logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

$failed = New-Object System.Collections.Generic.List[string]
$summary = New-Object System.Collections.Generic.List[string]

foreach ($case in $cases) {
    $romPath = Resolve-RomPath -Explicit $case.explicit -Candidates $case.candidates
    if (-not $romPath) {
        throw "ROM not found for case '$($case.Name)'. Provide -*RomPath arg or env var (NESIUM_ROM_KAGE/NESIUM_ROM_GIMMICK/NESIUM_ROM_NEKKETSU)."
    }

    Write-Host ""
    Write-Host "=== [$($case.Name)] $romPath ==="

    $logPath = Join-Path $logDir ("{0}.log" -f $case.Name)

    $cmd = @(
        "run", "--with", "numpy", "python", "tools/analyze_audio_alignment.py",
        "--rom", $romPath,
        "--start-frame", [string]$StartFrame,
        "--end-frame", [string]$EndFrame,
        "--mesen-dll", $MesenDllPath,
        "--out-dir", $OutDir
    )

    $cmdLine = "uv " + (($cmd | ForEach-Object { Quote-CmdArg $_ }) -join " ") + " 2>&1"
    $output = & cmd /c $cmdLine
    $exitCode = $LASTEXITCODE

    $output | Tee-Object -FilePath $logPath | Out-Host

    if ($exitCode -ne 0) {
        $failed.Add("$($case.Name): analyzer failed (exit=$exitCode)")
        continue
    }

    $rmse = Parse-Rmse -Lines $output
    $summary.Add("$($case.Name): RMSE=$rmse")

    if ($rmse -gt $MaxRmse) {
        $failed.Add("$($case.Name): RMSE=$rmse > MaxRmse=$MaxRmse")
    }
}

Write-Host ""
Write-Host "=== Summary ==="
foreach ($line in $summary) {
    Write-Host $line
}

if ($failed.Count -gt 0) {
    Write-Host ""
    Write-Host "=== Regression failures ==="
    foreach ($line in $failed) {
        Write-Host $line
    }
    exit 1
}

Write-Host ""
Write-Host "All audio regression cases passed."
exit 0
