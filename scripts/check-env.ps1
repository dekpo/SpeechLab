# M0 - environment check for AssistantCabinetAI-SpeechLab (Windows).
# Read-only: inspects the machine, installs nothing, sends nothing.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\check-env.ps1

$ErrorActionPreference = 'Continue'

function Test-Tool($name, $required, $why) {
    $c = Get-Command $name -ErrorAction SilentlyContinue
    $status = if ($c) { 'OK     ' } elseif ($required) { 'MISSING' } else { 'absent ' }
    $where = if ($c) { $c.Source } else { $why }
    '{0} {1,-8} {2}' -f $status, $name, $where
}

"== Machine"
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
"CPU   : $($cpu.Name) ($($cpu.NumberOfCores) cores / $($cpu.NumberOfLogicalProcessors) threads)"
"RAM   : $([math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1)) GB"
"OS    : $((Get-CimInstance Win32_OperatingSystem).Caption) $([Environment]::OSVersion.Version)"
"Arch  : $env:PROCESSOR_ARCHITECTURE"
"GPU   : $((Get-CimInstance Win32_VideoController | ForEach-Object Name) -join '; ')"

"`n== Toolchain"
Test-Tool node  $true  'needed for Vite/React'
Test-Tool pnpm  $false 'npm can be used instead'
Test-Tool rustc $true  'needed for Tauri'
Test-Tool cargo $true  'needed for Tauri'
Test-Tool cmake $true  'needed to build whisper.cpp / whisper-rs-sys (winget install Kitware.CMake)'
Test-Tool clang $true  'libclang needed by bindgen in whisper-rs-sys (winget install LLVM.LLVM)'
Test-Tool python $false 'optional: benchmark / dataset scripts'
Test-Tool ffmpeg $false 'optional: audio conversion for datasets'
Test-Tool git   $true  'needed to fetch sources'

"`n== MSVC / Windows SDK"
$msvc = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft Visual Studio\*\*\VC\Tools\MSVC\*" -ErrorAction SilentlyContinue | Select-Object -Last 1
if ($msvc) { "OK      MSVC $($msvc.Name)" } else { "MISSING MSVC Build Tools (C++ workload)" }
$sdk = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Include" -ErrorAction SilentlyContinue | Select-Object -Last 1
if ($sdk) { "OK      Windows SDK $($sdk.Name)" } else { "MISSING Windows 10/11 SDK" }

"`n== WebView2 runtime"
$wv = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft\EdgeWebView\Application" -ErrorAction SilentlyContinue | Where-Object Name -Match '^\d' | Select-Object -Last 1
if ($wv) { "OK      WebView2 $($wv.Name)" } else { "MISSING WebView2 runtime" }

"`n== Rust"
rustc -vV | Select-String 'host'
rustup target list --installed 2>$null

"`n== Audio input devices"
Get-CimInstance Win32_SoundDevice | ForEach-Object { "$($_.Status)  $($_.Name)" }
