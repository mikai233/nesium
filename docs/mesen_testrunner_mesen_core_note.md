# Mesen `testrunner` and `MesenCore.dll` update note (temporary)

## Problem
When running:

```powershell
Mesen.exe --testRunner ...
```

changes made in `Mesen2/Core` or `Mesen2/InteropDLL` may appear to be ignored.
This is usually seen as:

- new trace files not generated
- old behavior still present
- `C:\Users\<user>\Documents\Mesen2\MesenCore.dll` reverting after each run

## Root cause
`testrunner` startup calls `DependencyHelper::ExtractNativeDependencies(...)`.
It extracts **embedded** `Dependencies.zip` from `Mesen.dll`/`Mesen.exe` into Mesen Home.

So, manually copying `MesenCore.dll` to Home is not persistent:
- next `testrunner` run can overwrite it with the embedded version.

## Correct update workflow
1. Build core changes (InteropDLL/Core).
2. Build `UI` target so prebuild regenerates `Dependencies.zip` and re-embeds it.
3. Run `testrunner` (it will then extract the updated core to Home).

Recommended command:

```powershell
"C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe" `
  Mesen2/Mesen.sln /t:UI /p:Configuration=Release /p:Platform=x64 /m
```

## Quick verification
Check that Home core matches the newly built one:

```powershell
Get-Item `
  F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\MesenCore.dll, `
  C:\Users\dream\Documents\Mesen2\MesenCore.dll `
| Select-Object FullName,Length,LastWriteTime
```

If they differ after a run, you are likely still executing an older embedded `Dependencies.zip`.

## Save-data contamination pitfall (important)
When running `--testRunner`, Mesen can load existing `.sav` by ROM base name from:

- `C:\Users\<user>\Documents\Mesen2\Saves`

If you compare against NESium without loading matching save data, this can create false mismatches
(e.g. mapper/IRQ divergence caused by WRAM state differences, not by mapper logic).

### Reliable ways to avoid contamination
1. Use a fresh ROM filename per run (recommended for one-off traces).
2. Or delete the matching `.sav` before each Mesen run.

PowerShell helper:

```powershell
$rom = "F:/CLionProjects/nesium/target/compare/roms/kirby_clean_latest.nes"
$base = [System.IO.Path]::GetFileNameWithoutExtension($rom)
$sav = "C:/Users/dream/Documents/Mesen2/Saves/$base.sav"
Remove-Item $sav -ErrorAction SilentlyContinue
```

Then run Mesen/NESium once on the same ROM path and compare logs.

## How to run Lua trace scripts reliably
For headless trace generation, prefer `dotnet Mesen.dll` instead of launching `Mesen.exe` directly.

### Why
- `Mesen.exe` can return immediately as a GUI process and leave background instances.
- In `testrunner`, Lua `io/os` is sandboxed by default; scripts using `io.open(...)` will not write logs unless explicitly enabled.

### Recommended command template
```powershell
$env:NESIUM_MESEN_MMC3_TRACE_PATH = "F:/CLionProjects/nesium/target/compare/mmc3_batch/051_mesen_replay.log"
$env:NESIUM_MESEN_MMC3_TRACE_FRAMES = "240"

dotnet F:/CLionProjects/nesium/Mesen2/bin/win-x64/Release/Mesen.dll `
  --debug.scriptWindow.allowIoOsAccess=true `
  --timeout=300 `
  --testRunner F:/CLionProjects/nesium/target/compare/scripts/mesen_trace_mmc3_details.lua `
  "F:/nesrom/哈利传奇(中文).nes"
```

### Optional cleanup before rerun
```powershell
Get-Process Mesen -ErrorAction SilentlyContinue | Stop-Process -Force
```

### Quick sanity check (probe script)
```powershell
dotnet F:/CLionProjects/nesium/Mesen2/bin/win-x64/Release/Mesen.dll `
  --debug.scriptWindow.allowIoOsAccess=true `
  --testRunner F:/CLionProjects/nesium/target/compare_probe.lua `
  F:/nesrom/1943.nes
```
Expected output file:
- `F:/CLionProjects/nesium/target/compare/mesen_lua_probe.txt` containing `probe_ok`.
