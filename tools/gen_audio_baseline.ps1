param(
    [Parameter(Mandatory = $true)]
    [string]$RomPath,

    [Parameter(Mandatory = $true)]
    [int]$EndFrame,

    [int]$StartFrame = 0,

    [Parameter(Mandatory = $true)]
    [string]$OutFile,

    [string]$InputFile = "",
    [string]$InputEventsCsv = "",
    [int]$InputFrameOffset = 0,

    [string]$MesenDllPath = "F:\CLionProjects\nesium\Mesen2\bin\win-x64\Release\Mesen.dll",
    [string]$LuaScriptPath = "F:\CLionProjects\nesium\tools\mesen_record_audio.lua",
    [string]$WorkDir = "F:\CLionProjects\nesium\target\audio_baseline",
    [int]$SampleRate = 48000,
    [int]$TimeoutSec = 900,
    [switch]$AllowEmptyCapture,
    [switch]$NoCleanPersistentState
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

function Get-WavDataChunk {
    param([string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 12) {
        throw "WAV too small: $Path"
    }

    $audioFormat = $null
    $channels = $null
    $sampleRate = $null
    $bitsPerSample = $null
    $dataChunk = $null

    $i = 12
    while ($i + 8 -le $bytes.Length) {
        $id0 = [char]$bytes[$i]
        $id1 = [char]$bytes[$i + 1]
        $id2 = [char]$bytes[$i + 2]
        $id3 = [char]$bytes[$i + 3]
        $chunkId = "${id0}${id1}${id2}${id3}"
        $chunkSize = [BitConverter]::ToInt32($bytes, $i + 4)
        if ($chunkSize -lt 0) {
            throw "Invalid WAV chunk size at offset $i"
        }
        $dataStart = $i + 8
        $dataEnd = $dataStart + $chunkSize
        if ($dataEnd -gt $bytes.Length) {
            throw "WAV chunk exceeds file length at offset $i"
        }

        if ($chunkId -eq "fmt ") {
            if ($chunkSize -lt 16) {
                throw "Invalid WAV fmt chunk size: $chunkSize"
            }
            $audioFormat = [BitConverter]::ToUInt16($bytes, $dataStart + 0)
            $channels = [BitConverter]::ToUInt16($bytes, $dataStart + 2)
            $sampleRate = [BitConverter]::ToUInt32($bytes, $dataStart + 4)
            $bitsPerSample = [BitConverter]::ToUInt16($bytes, $dataStart + 14)
        } elseif ($chunkId -eq "data") {
            $dataChunk = New-Object byte[] $chunkSize
            [Array]::Copy($bytes, $dataStart, $dataChunk, 0, $chunkSize)
        }

        $i = $dataEnd
        if (($chunkSize % 2) -eq 1) {
            $i++
        }
    }

    if ($null -eq $dataChunk) {
        throw "WAV data chunk not found: $Path"
    }
    if ($null -eq $audioFormat -or $null -eq $channels -or $null -eq $sampleRate -or $null -eq $bitsPerSample) {
        throw "WAV fmt chunk not found: $Path"
    }

    [PSCustomObject]@{
        AudioFormat = [int]$audioFormat
        Channels = [int]$channels
        SampleRate = [int]$sampleRate
        BitsPerSample = [int]$bitsPerSample
        DataChunk = $dataChunk
    }
}

function Get-Sha1Base64 {
    param([byte[]]$Bytes)
    $sha1 = [System.Security.Cryptography.SHA1]::Create()
    try {
        $hash = $sha1.ComputeHash($Bytes)
        return [Convert]::ToBase64String($hash)
    } finally {
        $sha1.Dispose()
    }
}

if (-not (Test-Path $RomPath)) { throw "ROM not found: $RomPath" }
if (-not (Test-Path $MesenDllPath)) { throw "Mesen.dll not found: $MesenDllPath" }
if (-not (Test-Path $LuaScriptPath)) { throw "Lua script not found: $LuaScriptPath" }
if ($InputFile -and -not (Test-Path $InputFile)) { throw "input file not found: $InputFile" }
if ($StartFrame -lt 0) { throw "StartFrame must be >= 0" }
if ($EndFrame -le $StartFrame) { throw "EndFrame must be > StartFrame" }

if (-not $NoCleanPersistentState) {
    $mesenHome = Join-Path $env:USERPROFILE "Documents\\Mesen2"
    $romBase = [System.IO.Path]::GetFileNameWithoutExtension($RomPath)
    $savePath = Join-Path $mesenHome "Saves\\$romBase.sav"
    $stateGlob = Join-Path $mesenHome "SaveStates\\$romBase*.mss"
    Remove-Item $savePath -ErrorAction SilentlyContinue
    Get-ChildItem -Path $stateGlob -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue
}

$inputEvents = @()
if (-not [string]::IsNullOrWhiteSpace($InputFile)) {
    $inputEvents = Get-Content $InputFile |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) -and -not $_.Trim().StartsWith('#') }
} elseif (-not [string]::IsNullOrWhiteSpace($InputEventsCsv)) {
    $inputEvents = $InputEventsCsv.Split(',') |
        ForEach-Object { $_.Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
}

if ($InputFrameOffset -ne 0 -and $inputEvents.Count -gt 0) {
    $shifted = New-Object System.Collections.Generic.List[string]
    foreach ($token in $inputEvents) {
        $parts = $token.Split(':')
        if ($parts.Length -lt 2 -or $parts.Length -gt 3) {
            $shifted.Add($token)
            continue
        }

        [int]$frame = 0
        if (-not [int]::TryParse($parts[0], [ref]$frame)) {
            $shifted.Add($token)
            continue
        }

        $newFrame = $frame + $InputFrameOffset
        if ($newFrame -lt 0) {
            $newFrame = 0
        }
        $parts[0] = [string]$newFrame
        $shifted.Add(($parts -join ':'))
    }
    $inputEvents = $shifted
}

$inputEventsCsv = ($inputEvents -join ',')

New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$wavPath = Join-Path $WorkDir "mesen_audio_capture.wav"
$logPath = Join-Path $WorkDir "mesen_audio_capture.log"
if (Test-Path $wavPath) { Remove-Item $wavPath -Force }

$env:NESIUM_MESEN_AUDIO_WAV_OUT = $wavPath
$env:NESIUM_MESEN_AUDIO_START_FRAME = [string]$StartFrame
$env:NESIUM_MESEN_TRACE_FRAMES = [string]$EndFrame
$env:NESIUM_MESEN_INPUT_EVENTS = $inputEventsCsv

$mesenCmd = "dotnet `"$MesenDllPath`" --audio.sampleRate=_$SampleRate --debug.scriptWindow.allowIoOsAccess=true --timeout=$TimeoutSec --testRunner `"$LuaScriptPath`" `"$RomPath`" 2>&1"
$output = & cmd /c $mesenCmd
$output | Set-Content $logPath
if ($LASTEXITCODE -ne 0) {
    throw "Mesen testrunner failed with exit code $LASTEXITCODE. See $logPath"
}

$hash = $null
$sampleCount = 0
if (-not (Test-Path $wavPath)) {
    if (-not $AllowEmptyCapture) {
        throw "audio wav output not found: $wavPath"
    }
    $hash = Get-Sha1Base64 -Bytes (New-Object byte[] 0)
    $sampleCount = 0
} else {
    $wav = Get-WavDataChunk -Path $wavPath
    if ($wav.AudioFormat -ne 1 -or $wav.BitsPerSample -ne 16 -or $wav.Channels -ne 2) {
        throw "Unexpected WAV format (expect PCM16 stereo): format=$($wav.AudioFormat) bits=$($wav.BitsPerSample) channels=$($wav.Channels)"
    }
    if ($wav.SampleRate -ne $SampleRate) {
        throw "Unexpected WAV sample rate: $($wav.SampleRate) (expected $SampleRate)"
    }

    $hash = Get-Sha1Base64 -Bytes $wav.DataChunk
    $sampleCount = [int]($wav.DataChunk.Length / 2)
}

$outDir = Split-Path -Parent $OutFile
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
@(
    "# start_frame end_frame pcm16le_sha1 sample_count",
    "# sample_rate=$SampleRate channels=2 bits=16",
    "$StartFrame $EndFrame $hash $sampleCount"
) | Set-Content $OutFile

Write-Host "Audio baseline written: $OutFile"
Write-Host "data_sha1=$hash sample_count=$sampleCount"
