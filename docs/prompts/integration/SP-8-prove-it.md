# SP-8 launcher: Speech 8, prove it (validation, packaging, release gate)

**Tab name: "Speech 8 — Prove it".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-validation`, from `kb/integration`.

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md`, all `docs/test-reports/speech-programme/lots/*.md`, SpeechLab `docs/SPEECH_ENGINE_EVALUATION.md` (sections 7 and 8) and `docs/prompts/06-packaging-validation.md` (the M7 procedure).

**Written before the earlier phases ran: re-check everything.**

## Goal

Replace "it works on my machine" by evidence, as SpeechLab M7 did, and decide what ships.

## Preconditions (stop if not met)

The speech phases the owner chose are accepted. The practice PC (or a machine of the same class) is available for the
performance run. A quiet-machine check passes (SpeechLab's `bench preflight` rule; never publish figures from a forced run).

## Tasks

1. **Performance on the practice PC**: re-run the SP-0 probe with the installed engine and model; dictation latency for 5 s, 10 s and 20 s utterances; peak memory; behaviour during an Analyse pass (overlap); quote the background load and the 10 to 20 % margin. Compare with the budgets of plan section 4 and say plainly if they are missed.
2. **Offline proof**: with SpeechLab's `offline_proof_rule.ps1` pattern (a firewall rule blocking the application and the sidecar, created by the owner, who must also remove it), show that dictation (and read-aloud, if built) still works and that a download attempt fails. Sample the application's process tree's network endpoints as M7 did; state the web-view runtime exception honestly (SpeechLab I-066).
3. **Packaging**: wire the sidecar and the model manifest into the Windows installer using the method the SP-2 spike proved (a Windows-only overlay or a CI fetch step on both platforms); keep `cargo test --lib` green on `windows-latest` **and** `macos-latest`. Install silently into a scratch folder, run the packaged-app checks (adapt `ui_test_packaged.mjs`/`ui_test_readalong.mjs` to the AssistantCabinetAI origin and ports), microphone prompt and persistence in the release origin, first start without a model, uninstall (never run the interactive uninstaller without moving the real data folders first, as SpeechLab learned).
4. **Invariants table**: SI1 to SI12, each with its test name and its last result, in `docs/SPEECH.md`.
5. **Credits and licences**: the notices for each shipped model and component (CC BY, BSD-3, MIT, Apache, and the GPL notice if a phonemizer sidecar ships); a credits view if the owner approves (Q-09); `models/LICENSES.md` complete.
6. **Accuracy check with the real speakers**: the pilot GP's voice and microphone, fictional sentences with names and drug names from the fictional fixtures (`fixtures/gp-sandbox/` or the KB sandbox if it exists); report WER with the number of sentences and speakers and its interval; **no accent claim**.
7. **Release notes** for speech: "Windows only. macOS not validated and on hold." Update the status file, `docs/ROADMAP.md`, `docs/TROUBLESHOOTING.md`.
8. A short **handover** for maintenance: how to add an engine, how to update a model, how to switch speech off, what is known to be weak (SpeechLab's list), and what would reopen SpeechLab (measuring a new engine).

## Out of scope

macOS. New features. Changes to the KB.

## Final message (French)

Verdict by item (met / missed with the figure), what ships, what does not, the licence and credits status, the git block, and the list of NOT VERIFIED items that remain (macOS, clean machine, signing, SmartScreen).
