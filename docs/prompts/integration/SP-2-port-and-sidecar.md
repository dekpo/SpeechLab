# SP-2 launcher: Speech 2, speech-to-text port and the whisper.cpp sidecar (backend only)

> **GATE (D-050): ON HOLD.** Read `C:/Users/elise/Documents/CURSOR/SpeechLab/docs/integration/STATUS.md` first. If it does not say `STATUS: RESUMED`, **do not run this phase**: stop and tell the owner (in French) that the speech integration waits for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`). This launcher is draft v1 (2026-10-09); its facts and steps will be refreshed by that pass.

**Tab name: "Speech 2 — Port and whisper sidecar".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-stt-port`, from `kb/integration`.

Drag into the tab: `00-MASTER.md`, this file, `02-needs-and-structure-analysis.md` (sections 4, 5, 9), and AssistantCabinetAI's `docs/SPEECH.md`.

## Goal

The engine-independent backend of dictation: a `SpeechToTextProvider` port, a deterministic fake, the whisper.cpp sidecar
adapter, the private temp-audio lifecycle, machine-coded errors and three Tauri commands. **No user interface.**

## Preconditions (stop if not met)

- SP-1 accepted: `docs/SPEECH.md` and the speech decision section exist and Q-02, Q-03, Q-04 are answered.
- The probe (SP-0) allows whisper.cpp as the first engine, **or** the owner says "build the port anyway" (the port and the fake are engine-independent).
- `git status` clean of foreign work in the files you will edit (master section 6).

## Read first

`docs/SPEECH.md`; `apps/desktop/src-tauri/src/ocr/{mod.rs,tesseract.rs}` (the pattern), `src/cancellation.rs`, `src/error.rs`, `src/commands.rs` (`sidecar_search_roots` and its tests), `src/settings.rs`, `binaries/README.md`, `.github/workflows/ci.yml`; in SpeechLab: `src-tauri/src/speech/{whisper_cpp.rs,provider.rs,types.rs,wav.rs,error.rs}` and `scripts/build-whisper-cpp.ps1`, `scripts/stage-whisper-sidecar.ps1`.

## Tasks

1. `src-tauri/src/speech/`: `mod.rs` (types, `SpeechError` to `AppError` machine codes), `stt.rs` (the trait; request with locale `fr-FR`/`en-US`, a PCM or temp-WAV handle, an optional short hint list that defaults empty; result with text, audio and processing milliseconds, engine id and version), `fake.rs`, `temp_audio.rs` (a private folder under `app_local_data_dir()`, created per run, files removed in `Drop`, on error and on cancel, plus a sweep at start-up), `whisper_cli.rs` (argument building, language mapping from the locale by lookup table, kill on cancel, no console window on Windows, locate the sidecar with the existing discovery, parse the plain text output, **never put hints on the command line in this phase**), `manifest.rs` only if SP-3 needs the type now. ASCII only, no French word, no `cfg`, no HTTP client.
2. Error codes in `error.rs` and both catalogues: `speech_engine_unavailable`, `speech_model_missing`, `speech_audio_too_short`, `speech_audio_too_long`, `speech_cancelled`, `speech_engine_failed`, `speech_locale_unsupported` (French and English sentences in the catalogues, none in Rust).
3. Commands `speech_status` (availability for a locale), `speech_transcribe`, `speech_cancel`, registered with one line each in `lib.rs`; wrappers in `src/lib/ipc.ts`. One run in flight; stop uses the `cancellation.rs` pattern and kills the child.
4. `scripts/fetch-speech-resources.ps1` (checksum-pinned download of the whisper.cpp sidecar build and DLLs into the git-ignored `binaries/` tree, or a documented call to SpeechLab's build script) and `binaries/README.md` additions in the Tesseract style. **Do not declare `externalBin` in `tauri.conf.json`.**
5. **Spike, report the result**: does Tauri 2 merge a `tauri.windows.conf.json` overlay on Windows only, so that `externalBin` can be declared there without breaking `cargo test --lib` on the macOS CI job? Try it on a scratch copy of the config; record VERIFIED or NOT VERIFIED with the evidence. Do not commit an overlay unless the owner says so.
6. Tests: contract tests against the fake (success, cancel, too short, too long, unsupported locale); temp-audio lifecycle (SI1) on success, error and cancel; `startup_sweep_removes_orphan_audio`; SI2 as a source scan; SI6 (the timing line holds no text); SI9; SI10 via the existing guards; SI11 with a real child process (a tiny test program or `ping`-like command, as SpeechLab's cancel test did); an `#[ignore]` test with the real sidecar and a short synthetic clip (generate it with a clear-licence voice or a tone-free fixture; no real recording).

## Out of scope

UI, settings screens, model download, KB, TTS, sherpa-onnx, hints to the engine, installer wiring.

## Acceptance

On this Windows machine the ignored test transcribes a short clip through the sidecar; cancelling kills the child and removes the
temp file (check the process list and the folder); `cargo test`, `pnpm run test`, `pnpm run build` green; the CI-relevant parts
(`cargo test --lib`) do not need any Windows-only file.

## Report and human test

`docs/test-reports/speech-programme/lots/sp-2-port-and-sidecar.md`; human test: a smoke test plus the printed output of the ignored test and a process-list check after a cancel (no visible UI).

## Final message (French)

Done, verified, not verified (macOS, the overlay spike), limits, the git block, and: "Prochain onglet: Speech 3 — Models and settings (`SP-3-models-and-settings.md`)."
