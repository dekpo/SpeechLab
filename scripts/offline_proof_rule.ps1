# Adds or removes a Windows Firewall rule that blocks ALL outbound traffic of the packaged app and of its
# whisper.cpp sidecar, for the offline proof of M7. It changes the system, so it is run by the owner
# (never by an agent). It asks for administrator rights by itself (a Windows prompt appears).
#
#   scripts\offline_proof_add.cmd       block (default install folder: %LOCALAPPDATA%\SpeechLabM7Test)
#   scripts\offline_proof_remove.cmd    remove the rule again
#   scripts\offline_proof_show.cmd      show whether the rule exists (no administrator rights needed)
#
# or: powershell -ExecutionPolicy Bypass -File scripts\offline_proof_rule.ps1 add [-InstallDir <folder>]
#
# Only the two named programs are blocked (not WebView2, which other applications use).
param(
    [Parameter(Mandatory = $true, Position = 0)][ValidateSet('add', 'remove', 'show')][string]$Action,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'SpeechLabM7Test')
)
$ErrorActionPreference = 'Stop'
$name = 'SpeechLabM7-offline-proof'

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($Action -ne 'show' -and -not $isAdmin) {
    # Relaunch elevated; keep the window open so that the result can be read.
    $argList = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-NoExit', '-File', "`"$PSCommandPath`"", $Action, '-InstallDir', "`"$InstallDir`"")
    Start-Process powershell -Verb RunAs -ArgumentList $argList
    return
}

switch ($Action) {
    'add' {
        foreach ($exe in 'speechlab.exe', 'whisper-cli.exe') {
            $p = Join-Path $InstallDir $exe
            if (-not (Test-Path $p)) { throw "$p not found" }
            New-NetFirewallRule -DisplayName $name -Direction Outbound -Action Block -Program $p -Profile Any | Out-Null
            "blocked outbound: $p"
        }
        "Done. Tell the assistant, then remove the rule with scripts\offline_proof_remove.cmd"
    }
    'remove' {
        Remove-NetFirewallRule -DisplayName $name -ErrorAction SilentlyContinue
        "rule '$name' removed"
    }
    'show' {
        $rules = Get-NetFirewallRule -DisplayName $name -ErrorAction SilentlyContinue
        if (-not $rules) { "no rule named $name" }
        $rules | ForEach-Object {
            $f = $_ | Get-NetFirewallApplicationFilter
            "{0} | enabled={1} | {2} | {3}" -f $_.DisplayName, $_.Enabled, $_.Action, $f.Program
        }
    }
}
