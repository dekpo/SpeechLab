# Issues

Record every bug, blocker, or surprising behavior. Keep resolved items.

| ID | Status | Area | Summary |
|---|---|---|---|
| I-001 | Resolved | scripts | `check-env.ps1` printed `MSVC` instead of the toolset version |
| I-002 | Open | environment | CMake and LLVM not installed (needed only for whisper.cpp, milestone M3) |
| I-003 | Open | licensing | espeak-ng GPL-3.0 in sherpa-onnx Piper/Kokoro TTS path (see D-004) |
| I-004 | Open | models | License unknown for French streaming zipformer / Kroko models (HF returned 401) and Piper `miro`, `tjiho` voices (model card 404) |
| I-005 | Open | environment | `python` is the Microsoft Store build; verify behavior when scripts are run from services/CI |
| I-006 | Resolved | dev environment | Port 1420 conflict with the AssistantCabinetAI desktop dev server |
| I-007 | Open | tooling | pnpm 12 added `minimumReleaseAgeExclude: vite@8.3.3` automatically |
| I-008 | Open | verification | `transcribe` command not yet exercised by a UI click (only unit-tested) |

---

### I-001 — MSVC version not printed
- Symptom: `OK MSVC MSVC`.
- Cause: glob stopped one level too high (`...\VC\Tools\MSVC` instead of `...\MSVC\*`).
- Fix: glob changed to `...\VC\Tools\MSVC\*`. Re-run on 2026-10-06: now prints `MSVC 14.44.35207`. VERIFIED.

### I-002 — CMake / LLVM missing
- Impact: blocks whisper.cpp builds (M3) only. `whisper-rs-sys` needs CMake **and** libclang (bindgen); a sidecar build of whisper.cpp needs only CMake.
- Action: ask the owner before installing (`winget install Kitware.CMake`, `winget install LLVM.LLVM`).

### I-003 — espeak-ng GPL-3.0
- See D-004. Needs legal review and alternatives analysis in M6/M8.

### I-004 — Unknown licenses
- Re-check with a browser or alternate source in M8; treat as NOT usable until confirmed.

### I-005 — Store Python
- Low priority; note for benchmark scripts.

### I-006 — Port 1420 already in use
- Symptom: `Error: Port 1420 is already in use`, Tauri reports `beforeDevCommand` failed.
- Cause: the AssistantCabinetAI desktop app runs its own Vite dev server on 1420.
- Fix: SpeechLab uses 1430 (D-008). Do not kill the other process.

### I-007 — pnpm release-age protection
- pnpm 12 wrote the exclusion for Vite 8.3.3 into `pnpm-workspace.yaml` on its own (it appears to be its protection against very recently published versions; exact behaviour not investigated). Review when upgrading dependencies; the file is committed on purpose so installs are reproducible.

### I-008 — Unclicked transcribe path
- Action: click Transcribe with any non-empty path in the dev window and confirm the mock result appears; record it here. Real coverage arrives with M2.
