# Integration of speech into AssistantCabinetAI: documents

> **STATUS: ON HOLD (D-050).** The owner will finish the Knowledge Base first, then relaunch a pass on the plan to adapt it.
> Read `STATUS.md` before using anything here. The documents `01`, `02`, `03` and the prompts are draft v1 of 2026-10-09.

Prepared 2026-10-09, after the SpeechLab final report (M8). SpeechLab is frozen (D-048); the next work happens in
AssistantCabinetAI, as an autonomous speech module that uses the Knowledge Base for the spelling and pronunciation of names,
organisations and medicines.

| File | What it is |
|---|---|
| `STATUS.md` | **The status board**: ON HOLD, why, how the integration resumes, where the hold is announced |
| `KB-PREREQUISITES.md` | What speech needs from the Knowledge Base (contracts P-1 to P-9), the conditions to resume (G-1 to G-7), what KB lots should not break, the facts to re-verify, what the owner can note meanwhile |
| `01-owner-draft-and-review.md` | The owner's statements of 2026-10-09 (macOS on hold, SpeechLab finished), the owner's draft plan verbatim, and its review: what to keep, change, add |
| `02-needs-and-structure-analysis.md` | Facts read in AssistantCabinetAI (F1 to F20), what SpeechLab brings, the reuse map (keep, rewrite, leave), the target structure, the performance and TTS questions, the KB interplay, invariants SI1 to SI12, tests, risks |
| `03-integration-plan.md` | The plan to submit step by step: decisions Q-01 to Q-09, the engine decision tree, budgets, phases SP-0 to SP-8, timing against the KB lots |
| `../prompts/integration/` | The master brief and one launcher per phase, for the other chat tabs |
| `DECISIONS_TO_TAKE.md`, `PRACTICE_PC_PROBE.md`, `PROBE_RESULTS.md`, `repair-pairs-candidate.json` | Produced by phase SP-0 (not yet written) |

Related: `docs/SPEECH_ENGINE_EVALUATION.md` (evidence), `docs/DECISIONS.md` D-047 to D-049, `docs/ISSUES.md` I-075, I-076,
`docs/MACOS_VALIDATION.md` (on hold).

Status of the claims here: the statements about AssistantCabinetAI are **READ** (observed in its files on 2026-10-09, remote
`main` = `f27505f`, `kb/integration` = `d215939`, local branch `feat/kb-store` with uncommitted lot 1 work); nothing in that
repository was modified. Statements marked INFERRED or NOT VERIFIED say what must be tested.
