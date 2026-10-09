# Note for the Knowledge Base tabs: speech is coming, keep these contracts stable

**Informational. Drag this file into a KB lot tab (especially lots 2, 3, 8, 9 and 11) next to the lot's own files.** It adds no
task to the lot. Written 2026-10-09 by the SpeechLab sessions; copy it into a local `docs/SESSION-*.md` of AssistantCabinetAI if
you prefer that convention (the SpeechLab sessions do not write into that repository).

## What you need to know

1. The owner will integrate **workstation-local speech-to-text (dictation) and text-to-speech (read-aloud)** into AssistantCabinetAI **after** the Knowledge Base is finished. The plan is on hold and will be re-baselined then (`C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\STATUS.md`).
2. **Do not build speech in a KB lot.** KB decision D7/D8 and the lot files already say the KB only offers read-only contracts (`surface`, `repair`, `spoken_form`). That stays true.
3. The speech module will consume exactly the contracts of `SESSION-KB-00-master.md` section 13 and lot 9. The full list of what it needs, and what it relies on in the client, is `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\KB-PREREQUISITES.md` (parts A and C). Read those two parts if your lot is 2, 3, 8, 9 or 11.

## What to do if your lot touches a speech-relevant contract

Add a short section **"Speech-relevant changes"** to your lot report (`docs/test-reports/knowledge-base-pass-1/lots/lot-NN-*.md`),
and one line to the "Notes passed between lots" of `SESSION-KB-STATUS.md`, whenever your lot:

- changes the signature, fields or semantics of `surface_forms`, `repair::propose`, `spoken_form`, `title_spoken`, or the pack format and loader;
- changes how `knowledge_mode` (`off` / `suggest` / `auto`) gates reads;
- changes who owns the composer `draft`, the per-answer action row, the Settings "Advanced" group, the CSP, the catalogue parity test, the language guard, `cancellation.rs`, or sidecar discovery;
- decides how medicine names enter the KB, or whether English gets a phonetic encoder;
- adds a pack for `title_spoken` or a `spoken_form` editing surface (lot 8).

If there is nothing to report, write "Speech-relevant changes: none". These lines are the main input of the speech re-baseline.

## What not to do because of this note

Do not add speech code, a speech setting, a microphone control or a model dependency. Do not change a KB invariant, a schema or
a lot plan for speech's sake: if you think a KB change would help speech, write it in the report as a question for the owner.
Keep the usual rules (git is owner-only, no AI attribution, language contract, Windows and macOS code without `cfg`).
