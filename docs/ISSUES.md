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
| I-008 | Resolved | verification | `transcribe` command exercised through the real window in M2 |
| I-009 | Mitigated | environment | Antivirus TLS interception breaks the sherpa-onnx-sys build download (`UnknownIssuer`) |
| I-010 | Open | licensing | espeak-ng (GPL-3.0) is linked into the STT-only binary with the default sherpa-onnx libs; no-tts build avoids it |
| I-011 | Open | engine | sherpa-onnx offline decode cannot be cancelled mid-run |
| I-012 | Open | benchmarking | Large run-to-run timing variance (same input 1306 ms vs 2651 ms) |
| I-013 | Open | engine | sherpa-onnx prints resampler/debug lines to the console |
| I-014 | Open | data | No verified-license French test audio yet; `fr.wav` license unstated (test use only) |

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
- Resolved in M2: transcription, install and error paths were driven in the real Tauri window (WebView2 DevTools protocol, local only). Results are in PROJECT_LOG.

### I-009 — TLS interception breaks the crate build download
- Symptom: `Failed to download sherpa-onnx archive ... invalid peer certificate: UnknownIssuer` during `cargo build`.
- Cause: Avast Web/Mail Shield re-signs HTTPS traffic with its own root CA (present in Windows, absent from rustls bundled roots).
- Workaround: `scripts/fetch-sherpa-libs.ps1` (curl.exe + official SHA-256) and git-ignored `src-tauri/.cargo/config.toml` setting `SHERPA_ONNX_ARCHIVE_DIR`. Runtime model downloads use native certs and work.
- Not done on purpose: disabling the antivirus or TLS verification.

### I-010 — espeak-ng in the STT binary
- See D-012. Needs legal review; mitigation path = no-tts libs for STT-only builds.

### I-011 — No mid-run cancellation for sherpa-onnx
- Options for later: run inference in a child process and kill it, or accept "cancel = ignore result". Decide in M4 when recordings get long.

### I-012 — Timing variance
- Cause not investigated (laptop CPU boost/thermal, background load, antivirus). M5 must run N repetitions, record median and spread, fix the power mode, and log system state.

### I-013 — Console noise from sherpa-onnx
- Cosmetic. Revisit if the crate exposes a log hook.

### I-014 — Test audio licensing
- Plan: generate synthetic speech in M6 (TTS) and/or source public-domain or CC audio (e.g. LibriVox for fr/en) with provenance recorded. A Swiss French source is still unidentified.
