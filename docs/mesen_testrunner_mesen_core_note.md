# Mesen `testrunner` reliable workflow

## Goal
After modifying `Mesen2/Core`, make sure `--testRunner` actually runs the new instrumented logic.

## Core problem
`testrunner` does **not** load `MesenCore.dll` from `Mesen2/bin/...` directly.

It starts from:

- `Mesen2/bin/win-x64/Release/Mesen.dll`

Then `Program.cs` calls:

- `DependencyHelper.ExtractNativeDependencies(ConfigManager.HomeFolder)`

That extracts the embedded `Mesen.Dependencies.zip` into Mesen Home, usually:

- `C:\Users\<user>\Documents\Mesen2`

The actual native core used at runtime is:

- `C:\Users\<user>\Documents\Mesen2\MesenCore.dll`

So if Home still contains an old `MesenCore.dll`, `testrunner` will run old logic even if `Core` already rebuilt successfully.

## Why this can still fail after rebuilding UI
Even after `UI` rebuild embeds the new `Dependencies.zip`, Home may still keep the old `MesenCore.dll`.

Reason:

- `DependencyHelper.ExtractNativeDependencies(...)` only overwrites when file `LastWriteTime` or `Length` differs
- extraction errors are swallowed by `catch { }`
- if Home `MesenCore.dll` is locked/in use, extraction can fail silently

This is the exact failure mode that causes:

- new trace file path not appearing
- old log path still being written
- source code and runtime behavior not matching

## Reliable workflow
### 1. Rebuild UI
This regenerates and re-embeds `Dependencies.zip`.

```powershell
"C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe" `
  Mesen2/Mesen.sln /t:UI /p:Configuration=Release /p:Platform=x64 /m
```

### 2. Kill any running Mesen process
Do this before syncing Home.

```powershell
Get-Process Mesen,dotnet -ErrorAction SilentlyContinue | Stop-Process -Force
```

### 3. Force-sync Home `MesenCore.dll`
Do **not** rely only on auto-extraction.

```powershell
$src = "F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\MesenCore.dll"
$dst = "C:\Users\dream\Documents\Mesen2\MesenCore.dll"
Copy-Item $src $dst -Force
```

### 4. Verify the Home core really contains your new marker
Use a unique string from your instrumentation, for example a new output path.

```powershell
$dll = "C:\Users\dream\Documents\Mesen2\MesenCore.dll"
$bytes = [System.IO.File]::ReadAllBytes($dll)
$text = [System.Text.Encoding]::ASCII.GetString($bytes)

$text.Contains("mesen_apu_endframe_state_v2.csv")
```

Expected:

- `True`

If it is `False`, `testrunner` will still run old logic.

## Running the test runner
Prefer `dotnet Mesen.dll`.

```powershell
dotnet F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\Mesen.dll `
  --debug.scriptWindow.allowIoOsAccess=true `
  --timeout=300 `
  --testRunner tools/mesen_dump_state.lua `
  "D:\Game\roms\nes\Sangokushi 2 - Haou no Tairiku (Japan).nes"
```

## Final verification
Check the trace file that only the new instrumentation can produce.

Example:

```powershell
Get-Item F:\CLionProjects\nesium\target\compare\mesen_apu_endframe_state_v2.csv
```

If this file appears and updates after the run, you are executing the new instrumented logic.

## Fast diagnosis checklist
If Mesen still looks like it is running old code, check in this order:

1. `Release\MesenCore.dll` contains the new marker string
2. `Release\Mesen.dll` embedded `Dependencies.zip` contains the new core
3. `Documents\Mesen2\MesenCore.dll` contains the same marker
4. no stale `Mesen` / `dotnet` process is locking Home files
5. the log file path is a brand new filename, not an old reused file

## Save-data pitfall
For ROM comparisons, also clear matching Mesen save data when needed:

```powershell
$rom = "D:\Game\roms\nes\Sangokushi 2 - Haou no Tairiku (Japan).nes"
$base = [System.IO.Path]::GetFileNameWithoutExtension($rom)
$sav = "C:\Users\dream\Documents\Mesen2\Saves\$base.sav"
Remove-Item $sav -ErrorAction SilentlyContinue
```

Otherwise you can get false mismatches caused by stale WRAM/save state, not by emulator logic.
