# Builds whisper.cpp (CLI) from source with CMake + MSVC, CPU only.
# Source: official repository, pinned tag. Output: vendor\whisper.cpp\build\bin\whisper-cli.exe
#
# Prerequisites: CMake (winget install Kitware.CMake), MSVC Build Tools (C++ workload), git.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-whisper-cpp.ps1

$ErrorActionPreference = 'Stop'
$tag  = 'v1.9.4'
$root = Split-Path -Parent $PSScriptRoot
$src  = Join-Path $root 'vendor\whisper.cpp'

if (-not (Test-Path $src)) {
    git clone --depth 1 --branch $tag https://github.com/ggml-org/whisper.cpp.git $src
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
}

# CMake may not be on PATH in an already-open shell right after installation.
$cmake = (Get-Command cmake -ErrorAction SilentlyContinue).Source
if (-not $cmake) {
    $cmake = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Recurse -Filter cmake.exe -ErrorAction SilentlyContinue |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $cmake) { throw "cmake not found; install it with: winget install Kitware.CMake" }

$vcvars = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft Visual Studio\*\*\VC\Auxiliary\Build\vcvars64.bat" -ErrorAction SilentlyContinue |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $vcvars) { throw "vcvars64.bat not found; install the MSVC C++ build tools" }

$build = Join-Path $src 'build'
$cmakeArgs = '-S "{0}" -B "{1}" -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF -DWHISPER_SDL2=OFF -DGGML_CUDA=OFF -DGGML_VULKAN=OFF' -f $src, $build
$cmd = 'call "{0}" >nul && "{1}" {2} && "{1}" --build "{3}" --config Release' -f $vcvars, $cmake, $cmakeArgs, $build
cmd /c $cmd
if ($LASTEXITCODE -ne 0) { throw "whisper.cpp build failed" }

$exe = Join-Path $build 'bin\whisper-cli.exe'
if (-not (Test-Path $exe)) { throw "whisper-cli.exe not produced" }
"OK  $exe"
