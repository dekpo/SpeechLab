# Issues

Record every bug, blocker, or surprising behavior. Keep resolved items.

| ID | Status | Area | Summary |
|---|---|---|---|
| I-001 | Resolved | scripts | `check-env.ps1` printed `MSVC` instead of the toolset version |
| I-002 | Open | environment | CMake and LLVM not installed (needed only for whisper.cpp, milestone M3) |
| I-003 | Open | licensing | espeak-ng GPL-3.0 in sherpa-onnx Piper/Kokoro TTS path (see D-004) |
| I-004 | Open | models | License unknown for French streaming zipformer / Kroko models (HF returned 401) and Piper `miro`, `tjiho` voices (model card 404) |
| I-005 | Open | environment | `python` is the Microsoft Store build; verify behavior when scripts are run from services/CI |

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
