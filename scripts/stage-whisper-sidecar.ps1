# Stages the whisper.cpp command line tool for the Tauri bundler (sidecar).
#
# Tauri's externalBin needs the file name to carry the target triple, and the four shared
# libraries that whisper-cli.exe loads at start-up must sit next to it. They are copied here
# from the git-ignored build in vendor/ (see scripts/build-whisper-cpp.ps1) into the
# git-ignored folder src-tauri/binaries/. Run it before `pnpm tauri build` and before
# `pnpm tauri dev` (the bundler configuration refuses to start without these files).
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\stage-whisper-sidecar.ps1

$ErrorActionPreference = 'Stop'
$root   = Split-Path -Parent $PSScriptRoot
$bin    = Join-Path $root 'vendor\whisper.cpp\build\bin'
$out    = Join-Path $root 'src-tauri\binaries'
$triple = 'x86_64-pc-windows-msvc'

$exe  = Join-Path $bin 'whisper-cli.exe'
$dlls = 'ggml.dll', 'ggml-base.dll', 'ggml-cpu.dll', 'whisper.dll'

if (-not (Test-Path $exe)) { throw "whisper-cli.exe not found; run scripts\build-whisper-cpp.ps1 first" }
foreach ($d in $dlls) {
    if (-not (Test-Path (Join-Path $bin $d))) { throw "$d not found next to whisper-cli.exe in $bin" }
}

New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item $exe (Join-Path $out "whisper-cli-$triple.exe") -Force
foreach ($d in $dlls) { Copy-Item (Join-Path $bin $d) (Join-Path $out $d) -Force }

Get-ChildItem $out | Select-Object Name, Length | Format-Table -AutoSize
"OK  staged in $out"
