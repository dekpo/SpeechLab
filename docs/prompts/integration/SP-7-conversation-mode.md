# SP-7 launcher: Speech 7, spoken conversation (OPTIONAL; outline, refresh before use)

> **GATE (D-050): ON HOLD.** Read `C:/Users/elise/Documents/CURSOR/SpeechLab/docs/integration/STATUS.md` first. If it does not say `STATUS: RESUMED`, **do not run this phase**: stop and tell the owner (in French) that the speech integration waits for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`). This launcher is draft v1 (2026-10-09); its facts and steps will be refreshed by that pass.

**Tab name: "Speech 7 — Spoken conversation".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-conversation`, from `kb/integration`.

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md`, `01-owner-draft-and-review.md` (section 3.3), the SP-4 and SP-6 reports.

**This phase exists only if the owner decides yes (Q-07). Written before the earlier phases ran: re-check everything.**

## Why it is separate

The owner's draft step 5 asks for a mode where the user converses aloud with the chosen model. That removes the explicit send
that dictation keeps (SI3), can overlap the microphone with the voice, and loads a 2019 PC with recognition and synthesis at
once. KB decision D8 allows "read answers automatically" only as a possible later addition.

## Preconditions (stop if not met)

SP-4 and SP-6 in daily use by the owner or the pilot GP and accepted; Q-07 recorded as yes with the owner's conditions; a measured
CPU headroom on the practice PC for recognition plus synthesis.

## Task 1: a design note, for the owner's approval, before any code

`docs/test-reports/speech-programme/lots/sp-7-design.md`: the loop (push-to-talk first; optional hands-free later), end-of-speech
detection (the Silero VAD of SpeechLab, MIT, 0.6 MB, or push-to-talk only), the **confirmation before send** (a visible
countdown or a confirm key, configurable, never zero by default), **barge-in** (speaking over the voice stops it), **echo
control** (browser echo cancellation on for this mode, or the microphone muted while speaking; SpeechLab turned it off for
dictation on purpose), a global stop, the state display, what happens when the answer is long, when the model errors, when
the engine is slow, and how the loop provably **cannot reach file actions, fill plans or any outward action** (SI4). List the
privacy points: audio and transcripts never retained, names overheard. Ask the owner to approve or amend.

## Task 2 (after approval)

Implement exactly the approved design behind a setting that is **off by default**, in new files, reusing `useDictation` and the
read-aloud path. Tests: the loop's state machine with fakes (every transition, interrupt at every state), SI3 weakened only through
the explicit confirmation setting (a test that the default never auto-sends), SI4, SI11, no feedback loop when the voice plays
(an automated test with a fake audio source and the muted-microphone rule), CPU-load measurement on the practice PC.
Human test: a five-minute conversation by the owner with headphones, then without; interruptions; the stop.

## Final message (French)

Done, verified, not verified, limits (echo with loudspeakers), git block, then: "Prochain onglet: Speech 8 — Prove it."
