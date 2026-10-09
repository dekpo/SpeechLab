# SP-4 launcher: Speech 4, dictation in the chat

**Tab name: "Speech 4 — Dictation".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-dictation`, from `kb/integration` (after SP-3, and preferably after KB lot 7 or 8, because `Composer`, `ChatPanel`, `ipc.ts` and the catalogues are shared).

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md` (AssistantCabinetAI), the SP-2 and SP-3 reports.

## Goal

The owner's draft step 3: a microphone button in the chat; the transcript fills the editable box; the user sends explicitly.

## Preconditions (stop if not met)

SP-3 merged and accepted; a verified model installed on the test machine; `git status` shows no foreign uncommitted work in `Composer.tsx`, `ChatPanel.tsx`, `ipc.ts` or the catalogues (ask the owner to commit or finish the other tab first if it does).

## Read first

`docs/SPEECH.md`; `apps/desktop/src/components/{Composer.tsx,ChatPanel.tsx}`, `src/state/useChat.ts`, `src/lib/errors.ts`, `src/i18n/*`, `tauri.conf.json` (the CSP), `capabilities/default.json`; SpeechLab `src/audio/recorder.ts`, `src/audio/wav.ts`, `src/audio/` tests, `docs/DECISIONS.md` D-020 and D-045 (what broke with the CSP and why), `docs/ISSUES.md` I-024, I-025, I-028.

## Tasks

1. **Capture, ported**: `src/audio/{recorder.ts,levels.ts,wav.ts}` (getUserMedia, AudioWorklet, echo cancellation / noise suppression / auto gain **off**, 16 kHz mono PCM, level and clipping check). Ship the worklet as a **static file** (for example under `public/`) so the CSP needs no `blob:`; extend the CSP only as far as a test proves necessary and record the exact change in `docs/DECISIONS.md`. No clip store, no file save, nothing written to disk by the webview: the PCM goes to `speech_transcribe` and is dropped.
2. **State machine** `useDictation`: `unavailable | idle | requesting-permission | recording | transcribing | done | error`, with cancel in every state, a maximum duration from a named constant, a minimum duration, and the microphone track stopped on every exit. The machine codes of SP-2 map to catalogue keys; permission denied and no input device are front-end conditions with their own keys.
3. **Components** `MicButton`, `DictationStatus` (level meter while recording, elapsed time, `aria-live` status); one mount in `Composer`. Disabled with an explanation when `speech_status` is not ready; hidden when speech is switched off. Insertion at the cursor replacing a selection, one-step undo, focus back to the textarea, **no call to send**, Enter and the Send button unchanged.
4. A timing line (milliseconds and counts, no text) through the existing `write_timing_log` setting.
5. Keyboard: a documented shortcut to start and stop; Escape cancels recording or transcribing.
6. Documentation: `docs/CLIENT.md`, `docs/SPEECH.md`, `docs/TROUBLESHOOTING.md` (microphone blocked, no input device, the CSP trap).

## Out of scope

KB suggestions (SP-5), read-aloud, hints, conversation mode, macOS.

## Tests and acceptance

vitest: every transition of the state machine, cancel from each state, insertion and undo, `dictation_never_calls_send` (SI3), no import of any send/apply path (SI4), catalogue parity, error mapping. Rust: unchanged suites green. SI5 as a test that snapshots KB rows before and after a dictation if the KB tables exist.
Manual (the owner, with the pilot GP's voice and microphone, on fictional fixtures): the first permission prompt in the **development** origin and in the **release** origin (`tauri.localhost`; use SpeechLab's M7 DevTools procedure to drive it, or a real click), choice persisted after a restart, a dictated sentence with a name and a number, cancel while recording and while transcribing, speech off, model missing. Offline: the same with the network disabled.
Acceptance: press, speak, press, edit, send. With the engine missing the button explains and chat works.

## Report and human test

`docs/test-reports/speech-programme/lots/sp-4-dictation.md`; `human-tests/sp-4-dictation.md` (step list with expected results and tick boxes; include the five-minute smoke test).

## Final message (French)

Done, verified, not verified (macOS on hold; microphone click by a real hand if only driven by script), measured latency on this machine with the load, git block, then: "Prochain onglet: Speech 5 — Names and spelling (`SP-5-names-and-spelling.md`) quand les lots KB 2, 3, 6 et 9 sont fusionnés; sinon Speech 6 — Read aloud."
