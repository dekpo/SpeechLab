# SP-R launcher: Speech R, re-baseline the integration plan after the Knowledge Base

**Tab name: "Speech R — Re-baseline the plan".** Working folder: `C:\Users\elise\Documents\CURSOR\SpeechLab` (this pass edits
SpeechLab documents only and reads AssistantCabinetAI **read-only**). Branch: `docs/speech-rebaseline` (owner-run, from the current branch).

Drag into the tab: `00-MASTER.md`, this file, and from `docs/integration/`: `STATUS.md`, `KB-PREREQUISITES.md`, `01-owner-draft-and-review.md`,
`02-needs-and-structure-analysis.md`, `03-integration-plan.md`.

## Why this pass exists

On 2026-10-09 the owner decided to finish the Knowledge Base implementation first and then relaunch a pass on the integration
plan to adapt it (D-050). The v1 documents are a draft valid on 2026-10-09. This pass turns them into a plan that is true on the
day it runs. **This is the only speech launcher that may be run while `STATUS.md` says ON HOLD**, and only when the owner asks.

## Preconditions (check first; stop and report if not met)

- The owner says the KB implementation is finished, or finished enough, and says which lots are merged (condition G-1 of `KB-PREREQUISITES.md`). If the owner has not said so, ask; do not infer it from the repository.
- `git status` of AssistantCabinetAI is **not** to be changed by you in any way. You read it. If the working tree there has uncommitted work of another tab, note it and do not touch it.
- No benchmark is running (`tasklist`).

## Read first

`AGENTS.md` (SpeechLab rules: git is owner-only, no AI or vendor names, English docs, no invented numbers), `docs/integration/STATUS.md`,
`KB-PREREQUISITES.md` (all parts), `01`, `02`, `03` and `docs/prompts/integration/*` (the v1 launchers), `docs/SPEECH_ENGINE_EVALUATION.md` sections 2, 5, 6, 9, 10, `docs/DECISIONS.md` D-047 to D-050.
Then AssistantCabinetAI as it is **now**: start with the commands of `KB-PREREQUISITES.md` part D.

## Steps

1. **Ask the owner (in French, in one message)**: what is finished in the KB and what is not; whether `kb/integration` was merged into `main`; the macOS situation (still no Mac?); the practice PC (model, CPU, RAM, Windows version) and whether SP-0's probe was run; the answers already known to Q-01 to Q-09; the points of `KB-PREREQUISITES.md` part E; anything the owner learned while using the KB; whether the plan should be shortened (for example dictation only first). Wait for the answers before changing documents. Do not decide for the owner.
2. **Snapshot AssistantCabinetAI** (read-only): remote branches and tags, merged KB lots, `main` versus `kb/integration`, the KB status file and lot reports, **every "Speech-relevant changes" section** of the lot reports (the KB tabs were asked to write them, see `KB-TAB-NOTE.md`), new sections of `docs/DECISIONS.md`, changes to `AGENTS.md` and `.cursor/rules`, `Cargo.toml` dependencies, `tauri.conf.json` (CSP, bundle), CI workflow, `settings.rs` fields, the shape of `Composer`, `ChatPanel`, `MessageList`, `SettingsDialog`.
3. **Re-verify facts F1 to F20** of `02-needs-and-structure-analysis.md`. Write `docs/integration/REBASELINE-<date>.md` with a table: fact, status (unchanged / changed with the new value / obsolete), source file, and what the change does to the plan. Re-verify the KB contracts P-1 to P-9 against the **actual code and tests** (signatures, fields, behaviour), not the plans.
4. **Re-check SpeechLab-side assumptions** that depended on the KB: the reuse map (does the KB now have a normaliser, a pack loader or a spoken-form mechanism that replaces something proposed in `02` section 4?), the repair design against the contract fixtures, whether the candidate pairs of SP-0 exist and still fit.
5. **Write plan v2.** First archive v1: copy `01`, `02`, `03` and the v1 launchers to `docs/integration/archive/v1-2026-10-09/` (never delete them). Then update `02` (facts), `03` (decisions, phases, dependencies, branch, sizes) and the launchers `SP-0` to `SP-8` (and `00-MASTER.md` section 2 and 3), keeping the invariants SI1 to SI12 unless a fact forces a change (say why). Typical expected changes: the base branch (`main` if the KB gate is passed), SP-5 simplified or reordered because the real APIs exist, phases merged or removed, new constraints from the KB's final form. Mark every phase with its preconditions again.
6. **Decisions Q-01 to Q-09**: record the owner's answers; leave the open ones open with their trigger. Update the decision sheet if SP-0 produced one.
7. **Status**: do **not** edit the status line yourself. Propose in the chat to change `docs/integration/STATUS.md` to `STATUS: RESUMED` (with date and what changed) and wait for the owner's explicit yes; then make that single edit, update the history table, and update the banners listed in `STATUS.md` ("Where the hold is announced"): remove or change the hold text in `AGENTS.md` section 7, `docs/HANDOFF.md`, `README.md`, `docs/integration/README.md`, the headers of `01`/`02`/`03`, the prompts README, the master and the launchers. If the owner says "not yet", leave the hold and record what is missing.
8. Document: `docs/PROJECT_LOG.md` entry (Done / Verified / Failed or surprises / Not verified / Next), `docs/DECISIONS.md` (a decision recording the v2 plan and the answers), `docs/ISSUES.md` (new or closed), `docs/HANDOFF.md` sections 3 and 7, `README.md`. Mark claims VERIFIED or NOT VERIFIED.

## Out of scope

Any edit of AssistantCabinetAI. Running any speech phase. Building anything. New SpeechLab measurements (except what SP-0 already defines). Taking a decision that belongs to the owner.

## Acceptance

`REBASELINE-<date>.md` lists every fact with a status and a source; v1 is archived intact; v2 documents and launchers are consistent
with each other and with the repository as it is; the owner has answered or explicitly deferred each of Q-01 to Q-09; the status
line is changed only with the owner's yes.

## Final message to the owner (French)

What changed since 2026-10-09 (five lines), what the new plan looks like (phases and order), the decisions still open, the risks that
moved, the git block (branch, `git add` of explicit paths, one English commit line of at most 72 characters, no trailer, push),
and which tab to open next with which launcher.
