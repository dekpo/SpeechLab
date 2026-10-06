# Downloads the prebuilt sherpa-onnx static libraries into vendor/sherpa-onnx and writes the
# local (git-ignored) cargo override that points the crate build script at them.
#
# Why: the sherpa-onnx-sys build script downloads these libs with its own TLS stack, which
# does not trust the Windows certificate store. On machines where an antivirus or proxy
# intercepts TLS (e.g. Avast Web Shield) that download fails with "UnknownIssuer".
# curl.exe uses the Windows store, so it works there. See docs/ISSUES.md I-009.
#
# The archive is verified against the SHA-256 published by the official GitHub release.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\fetch-sherpa-libs.ps1

$ErrorActionPreference = 'Stop'
$version = '1.13.8'   # must match the sherpa-onnx crate version in src-tauri/Cargo.toml
$name    = "sherpa-onnx-v$version-win-x64-static-MT-Release-lib.tar.bz2"
$url     = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v$version/$name"
$root    = Split-Path -Parent $PSScriptRoot
$dir     = Join-Path $root 'vendor\sherpa-onnx'
$file    = Join-Path $dir $name

New-Item -ItemType Directory -Force $dir | Out-Null

$release = Invoke-RestMethod "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/v$version" -Headers @{ 'User-Agent' = 'speechlab' }
$asset = $release.assets | Where-Object name -eq $name
if (-not $asset -or -not $asset.digest) { throw "No published digest for $name" }
$expected = $asset.digest -replace '^sha256:', ''

if (-not (Test-Path $file) -or (Get-FileHash $file -Algorithm SHA256).Hash.ToLower() -ne $expected) {
    curl.exe -L --fail -sS -o $file $url
    if ($LASTEXITCODE -ne 0) { throw "download failed" }
}
$actual = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
if ($actual -ne $expected) { Remove-Item $file; throw "checksum mismatch: expected $expected, got $actual" }
"OK  $name  sha256=$actual"

$cargoDir = Join-Path $root 'src-tauri\.cargo'
New-Item -ItemType Directory -Force $cargoDir | Out-Null
@"
# Local, git-ignored (written by scripts/fetch-sherpa-libs.ps1).
[env]
SHERPA_ONNX_ARCHIVE_DIR = { value = "../vendor/sherpa-onnx", relative = true }
"@ | Set-Content -Encoding ascii (Join-Path $cargoDir 'config.toml')
"OK  wrote src-tauri\.cargo\config.toml"
