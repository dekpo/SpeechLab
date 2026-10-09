# SP-3 launcher: Speech 3, models, availability and settings

> **GATE (D-050): ON HOLD.** Read `C:/Users/elise/Documents/CURSOR/SpeechLab/docs/integration/STATUS.md` first. If it does not say `STATUS: RESUMED`, **do not run this phase**: stop and tell the owner (in French) that the speech integration waits for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`). This launcher is draft v1 (2026-10-09); its facts and steps will be refreshed by that pass.

**Tab name: "Speech 3 — Models and settings".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-models-settings`, from `kb/integration` (after SP-2 is merged there).

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md` (AssistantCabinetAI), the SP-2 report.

## Goal

The application knows whether speech can work on this machine and lets the user control it, **before** any microphone button exists.

## Preconditions (stop if not met)

SP-2 merged and accepted. Q-05 (how model files reach a practice machine) answered, at least for development. Q-01 answered if a CC BY model is on the table.

## Read first

`docs/SPEECH.md`; `src/settings.rs` (every field, the clamp-on-the-way-out idiom, the constants), `src/components/SettingsDialog.tsx` (the Advanced group), `src/state/useAppSettings.ts`, `src/i18n/catalogues.test.ts`; SpeechLab `src-tauri/src/speech/models.rs`, `download.rs` and `src-tauri/models-manifest.json` for the checksum-pinned manifest design (**port the design, not the code**: the guard forbids French literals and SpeechLab's code has them in places).

## Tasks

1. `resources/speech/models-manifest.json`: id, display key, languages, engine, size, SHA-256, licence, source URL, quantisation, expected memory (only measured values, else absent). Verification code in `speech/manifest.rs` (good, corrupted, missing file, wrong size) and a marker file written only after verification.
2. Install paths per Q-05: (a) a development download started by an explicit user action, native certificate store, checksum checked, no inference code involved; (b) side-load: the user points at a folder or drops the file in the models folder, the manifest verifies it. No audio ever uses this path (SI2 stays true: the installer is a separate, user-started act; keep it out of the speech inference module).
3. `models/LICENSES.md`: each pulled model's row, licence read from the exact card, date.
4. Settings with named defaults in `settings.rs`: `speech_enabled` (false until a model verifies), `stt_model`, `stt_language_override` (None follows the locale), `speech_hints` (false; no effect until SP-5), reserved `tts_*` fields only if SP-6 needs them (otherwise leave them out). A "Speech" block inside the Advanced group of `SettingsDialog.tsx`: a status line (ready, model missing, engine missing, locale unsupported), the install or side-load action, the on/off switch. All strings in both catalogues.
5. `speech_status` returns the full availability matrix; the front end uses it only to decide what to show.

## Out of scope

The microphone button, recording, KB, TTS, installer bundling.

## Tests and acceptance

Manifest verification cases; settings round trip, defaults and clamping; the availability matrix; SI9 (with no model the chat is unchanged); catalogue parity; SI2 (no HTTP client in the inference module). Acceptance: with no model the Settings block says so and nothing else changes; with a verified model it says ready; switching speech off restores today's behaviour.

## Report and human test

`docs/test-reports/speech-programme/lots/sp-3-models-and-settings.md`; human test: open Settings, read the status in each of the four situations (no model, corrupted model, valid model, speech off), and the owner's side-load of a real model file.

## Final message (French)

Done, verified, not verified, git block, then: "Prochain onglet: Speech 4 — Dictation (`SP-4-dictation.md`)." Remind the owner that SP-4 touches `Composer` and the catalogues and should follow KB lots 7 or 8.
