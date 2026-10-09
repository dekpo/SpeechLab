# Speech integration: status board (single source of truth)

**STATUS: ON HOLD** (set 2026-10-09 by the owner, D-050)

Every agent and every session must read this file before touching anything under `docs/integration/` or
`docs/prompts/integration/`, and before running any `SP-*` launcher. **If the line above does not say `STATUS: RESUMED`, do not
execute a speech phase: stop and tell the owner** (in French in the chat) that the integration is on hold and why.

## Why it is on hold

The owner decided on 2026-10-09 to **finish the implementation of the Knowledge Base (KB) in AssistantCabinetAI first**, and
then to **relaunch a pass on the integration plan to adapt it**. Reasons that were already visible in the analysis: speech
depends on KB lots 2, 3, 6 and 9 (phonetic key, packs, selection mapping, `surface` and `repair`), the KB edits the same shared
files, the KB branch model decides the base branch, and the facts read on 2026-10-09 will have moved.

## What the plan is while on hold

- `01-owner-draft-and-review.md`, `02-needs-and-structure-analysis.md`, `03-integration-plan.md` and `docs/prompts/integration/*`
  are **DRAFT v1, valid as of 2026-10-09** (AssistantCabinetAI remote `main` = `f27505f`, `kb/integration` = `d215939`, KB lot 0
  merged, lot 1 code done and uncommitted). They are **not to be executed as they stand** and must not be treated as current
  facts about AssistantCabinetAI. They are kept because the thinking (reuse map, invariants SI1 to SI12, decisions Q-01 to Q-09,
  corrections to the owner's draft, risks) is expected to survive; the facts and the phase details are expected to change.
- What is **not** on hold: SpeechLab itself stays frozen (D-048). macOS stays on hold (D-048, I-075).
- Optional and independent of the KB, the owner may run **SP-0** (practice-PC probe, decision sheet) at any time; it touches only
  SpeechLab and its results will be inputs of the re-baseline. It is not required before the re-baseline.

## How the integration resumes

1. The owner finishes the KB implementation (see the resume conditions in `KB-PREREQUISITES.md`, part B).
2. The owner opens a tab named **"Speech R — Re-baseline the plan"** and drags `docs/prompts/integration/00-MASTER.md` and
   `docs/prompts/integration/SP-R-rebaseline.md` into it.
3. That pass re-reads AssistantCabinetAI as it is then, re-verifies facts F1 to F20, collects the owner's new inputs, writes the
   plan **v2** and refreshed launchers, archives v1, and proposes the status change.
4. Only the owner sets the line above to `STATUS: RESUMED` (the agent proposes the edit; the owner confirms in the chat).
5. Then the owner submits the refreshed launchers one at a time.

## Information the owner gave for this hold (2026-10-09, recorded)

> "Je vais d'abord terminer l'implémentation de la Knowledge Base dans AssitantCabinetAI et ensuite nous relancerons une passe sur
> ce plan d'intégration pour l'adapter correctement."

English: the owner will first finish the Knowledge Base implementation in AssistantCabinetAI, then a pass will be relaunched on
the integration plan to adapt it properly. Future vibe-coding sessions and their agents must take this into account.

## Where the hold is announced (so no agent can miss it)

`AGENTS.md` section 7 (read first in every session), `docs/HANDOFF.md` banner, `README.md` banner, `docs/integration/README.md`,
the header of `01`, `02`, `03`, `docs/prompts/integration/README.md`, `00-MASTER.md` and every `SP-*` launcher, D-050 in
`docs/DECISIONS.md`, and the project memory of the assistant tooling. The drop-in note for the KB tabs is
`docs/prompts/integration/KB-TAB-NOTE.md`.

## History

| Date | Event |
|---|---|
| 2026-10-09 | M8 report; SpeechLab frozen, macOS on hold (D-048); integration analysis, draft plan v1 and prompts written (D-049 proposed) |
| 2026-10-09 | Owner: finish the KB first, then relaunch a pass to adapt the plan. Status set to ON HOLD (D-050); re-baseline launcher, KB prerequisites and KB tab note written |
