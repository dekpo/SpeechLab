# SP-5 launcher: Speech 5, names and spelling from the Knowledge Base

**Tab name: "Speech 5 — Names and spelling".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-kb-spelling`, from `kb/integration`.

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md`, `docs/SESSION-KB-00-master.md` (sections 3, 7, 13), `docs/SESSION-KB-04-surfaces.md` (lot 9), the SP-4 report, SpeechLab's `docs/integration/repair-pairs-candidate.json`.

**Written before the earlier phases ran: re-check every fact against the repository first and list what changed in the report.**

## Goal

The owner's central requirement: dictated proper names, organisations and medicines are understood and spelled better because
the speech module can read the Knowledge Base, **without ever writing to it and without leaving the user's selection**.

## Preconditions (stop if not met)

SP-4 accepted. KB lots **2** (normalisation, French phonetic key), **3** (ports, packs, resolver), **6** (selection mapping) and
**9** (`surface`, `repair`) merged into `kb/integration`; verify by reading the code (`knowledge/surface.rs`, `knowledge/repair.rs`)
and the KB status file, not by trusting this paragraph. If lot 9 is missing, stop: do not re-implement it.

## Tasks

1. `speech/hints.rs`: `trait SpeechHintSource` with an adapter over `knowledge::surface_forms(selection, limit, SpeechHints)` and `knowledge::repair::propose(transcript, selection, locale, ...)`; KB mode `off` returns an empty source; the speech module imports nothing else from `knowledge`.
2. **Repair-first**: after a transcription, call `propose`; the interface shows suggestion chips `heard -> display` (accept replaces the span, ignore leaves it, `Ask` lists the alternatives). Never a silent replacement; names hidden on screen by default follow the KB setting `knowledge_hide_names_in_suggestions` (decision D10: a screen can be seen by people in the waiting room).
3. **Contract fixtures**: place the SpeechLab candidate pairs (reviewed by you) into `tests/fixtures/knowledge/repair-fr.json` with the must-not-change pairs and run `repair` against them. Report precision (accepted correct), false suggestions, and misses. Do not tune the KB's thresholds here; if the fixtures expose a defect, report it to the owner for the KB tab.
4. **Optional engine hints**, behind `speech_hints` (default off): a short, selection-scoped list; **never passed on a command line** (`whisper-cli --prompt` is a command-line argument, visible in the process list). With the whisper.cpp sidecar the setting therefore has no effect in this phase; document why. A hint-capable path through stdin or an API belongs to a later engine adapter. Measure nothing you cannot measure honestly.
5. Settings text in plain French and English explaining what the option does and that names stay on this computer.
6. Documentation: `docs/SPEECH.md`, `docs/PRIVACY-AND-SECURITY.md`, KB status notes (cross-reference only; do not edit KB design).

## Tests and acceptance

`hints_never_leave_the_selection`, `kb_off_gives_no_hints`, SI5 (`dictating_does_not_change_the_knowledge_base`, row-hash snapshot before and after), SI6 (no hint or name in any log), SI12; repair against the fixture file including must-not-change pairs; vitest for the chips and their accessibility.
Human test (owner, fictional fixtures: `fixtures/gp-sandbox/`, or the KB sandbox if KB lot 11 already created it): dictate a misspelled surname and a drug name from the sandbox documents; the right suggestion appears; accept it; the KB tables are unchanged (use the KB's read-only inspection script); then dictate a sentence with no name and see no chip.

## Final message (French)

Done, verified (fixture precision), not verified, limits (one speaker's errors in the fixtures; French phonetic key only), git block, then: "Prochain onglet: Speech 6 — Read aloud (`SP-6-read-aloud.md`)."
