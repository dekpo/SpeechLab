# SP-1 launcher: Speech 1, contracts and decisions (documents only)

> **GATE (D-050): ON HOLD.** Read `C:/Users/elise/Documents/CURSOR/SpeechLab/docs/integration/STATUS.md` first. If it does not say `STATUS: RESUMED`, **do not run this phase**: stop and tell the owner (in French) that the speech integration waits for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`). This launcher is draft v1 (2026-10-09); its facts and steps will be refreshed by that pass.

**Tab name: "Speech 1 — Contracts and decisions".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `docs/speech-contracts`, created from `kb/integration` (decision Q-03; if the owner chose otherwise, follow the owner).

Drag into the tab: `00-MASTER.md`, this file, `03-integration-plan.md`, `02-needs-and-structure-analysis.md`, and the owner's filled `DECISIONS_TO_TAKE.md` (SpeechLab `docs/integration/`).

## Goal

Make the speech work allowed, specified and bounded **in the AssistantCabinetAI repository before any code**, exactly as KB lot 0
did for the KB. This phase changes documents and one rule file; it changes no source file.

## Preconditions (stop if not met)

- SP-0 done; the owner has answered at least Q-02 (process model), Q-03 (base branch) and Q-04 (macOS waiver) on the decision sheet. Q-01 may be "after the probe". Do not answer a question yourself.
- `git status`: if the tree holds uncommitted KB work of another tab, do not touch it (master section 6); documents you edit must not be among the files that work changes. List the overlap if any and ask.
- `kb/integration` exists and is the base (`git branch -a`).

## Read first

`AGENTS.md`, `CONTRIBUTING.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md` (especially "Reserved contracts", "Client UI", "Knowledge Base and the product direction" D7 to D10), `docs/CHAT-UX-ASSESSMENT.md` sections 6 and 7, `docs/PRIVACY-AND-SECURITY.md`, `docs/LANGUAGE-AND-LOCALE.md`, `docs/CLIENT.md`, `docs/ROADMAP.md`, `models/LICENSES.md`, `.cursor/rules/kb-programme.mdc`, `.cursor/rules/v0-sprint.mdc`, `docs/SESSION-KB-00-master.md` section 13, `docs/SESSION-KB-WORKFLOW.md`.

## Tasks

1. `docs/DECISIONS.md`: add a section "Speech: workstation-local dictation and read-aloud (date)". A table with: the owner's answers to Q-01 to Q-09 (open ones marked open with the trigger that closes them); the macOS waiver (what is waived, what is not: code stays `cfg`-free and compiles in CI, no macOS validation, no macOS claim); the process model (sidecar only); the two ports; the unused `/v1/audio/*` routes; the **revision of the voice rule** so that read-aloud of text already on screen with local voices is allowed (the text the assessment says must be amended before the code); invariants SI1 to SI12 with their test names; the first engine and how it is chosen (probe, decision tree of plan section 3); the KB relationship (repair-first, hints opt-in, never a command line); cross-reference to SpeechLab's report and D-047/D-048/D-049.
2. `docs/SPEECH.md`: the canonical, long-lived spec built from analysis sections 5 to 10 (layout, ports, flows, settings, machine codes, privacy table, packs, interface notes, test strategy). English. It is a specification, not a plan: no dates, no phase names except in one "delivery" paragraph.
3. Update, minimally and consistently: `docs/ARCHITECTURE.md` (ports and sidecars), `docs/PRIVACY-AND-SECURITY.md` (audio and transcripts, temp files, hint leakage, overheard read-aloud), `docs/LANGUAGE-AND-LOCALE.md` (speech locale mapping and rule packs as data), `docs/CLIENT.md` (mic button, read-aloud action), `docs/ROADMAP.md` (a speech track beside K-A to K-E, MAC-HOLD), `models/LICENSES.md` (a row per speech model considered, licence **read from the exact card now**, status "not pulled"), and the "Reserved contracts" paragraph of `docs/DECISIONS.md` (the LAN routes are not used).
4. `.cursor/rules/speech-programme.mdc`: the ritual and the invariants, modelled on `kb-programme.mdc`. Do not weaken any existing rule; only add. If `v0-sprint.mdc` needs one sentence so agents stop refusing speech, add that sentence and say so.
5. Create `docs/SESSION-SPEECH-STATUS.md` (local, git-ignored like the other SESSION files): phases SP-1 to SP-8 with state, branch, dependencies on KB lots, notes passed between phases. Mark SP-1 done at the end.
6. Re-check the facts of the analysis F1 to F20 against the repository **as it is today** and list any that changed (KB lots may have merged since 2026-10-09). Put the list in the phase report.

## Out of scope

Any source code, any dependency, any change to the KB design or invariants (propose to the owner), any SpeechLab edit.

## Tests and acceptance

Existing suites unchanged and green (nothing to add); `pnpm run test` still passes the language guard on documents it scans.
The owner can read `docs/SPEECH.md` and the decision section and answer yes, no or amend. Every claim about the repository cites a file.

## Report and human test

Report: `docs/test-reports/speech-programme/lots/sp-1-contracts.md`. Human test: "read and annotate", with a checklist of the ten
points of the decision table. 

## Final message (French)

Done, verified (which facts were re-checked, which changed), not verified, the open decisions that block SP-2, the git block, and:
"Prochain onglet: Speech 2 — Port and whisper sidecar (`SP-2-port-and-sidecar.md`)."
