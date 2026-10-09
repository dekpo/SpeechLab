# SP-0 launcher: Speech 0, freeze SpeechLab and probe the practice PC

> **GATE (D-050): ON HOLD.** The speech integration waits for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`; status in `C:/Users/elise/Documents/CURSOR/SpeechLab/docs/integration/STATUS.md`). **SP-0 is the one exception the owner may choose to run earlier**, because it only touches SpeechLab, is independent of the Knowledge Base, and its results (decision sheet, practice-PC probe, candidate repair pairs) are inputs of the re-baseline. Run it only if the owner asks for it explicitly. This launcher is draft v1 (2026-10-09).

**Tab name: "Speech 0 — Freeze and practice-PC probe".** Working folder of the tab: `C:\Users\elise\Documents\CURSOR\SpeechLab`
(this is the only speech phase that edits SpeechLab). Branch: `milestone/m9-integration-prep` (from the current branch, owner-run).

Drag into the tab: `00-MASTER.md`, this file, `docs/integration/03-integration-plan.md`, `docs/integration/02-needs-and-structure-analysis.md`.

## Read first (SpeechLab rules apply in this tab)

`AGENTS.md`, `docs/HANDOFF.md`, the last entries of `docs/PROJECT_LOG.md` (M8 and the integration-preparation entry), `docs/DECISIONS.md` (D-047 to D-049), `docs/ISSUES.md` (open items), `docs/SPEECH_ENGINE_EVALUATION.md` sections 2, 6, 9 and 10, `docs/integration/*.md`.
SpeechLab rules still hold: git is owner-only, no AI or vendor names anywhere in the repository, documentation in English in `docs/`, no simulated data, no private content (aggregate numbers only), `bench` needs a quiet machine and never `--force` for published figures, two benchmarks never at once.

## Goal

Close SpeechLab as a reference and a measurement bench, put the owner's decisions Q-01 to Q-09 on one sheet, and give the owner a
way to **measure the real practice PC** (a Windows PC installed in 2019, specification unknown) before any engine is chosen.

## Preconditions (stop if not met)

- The M8 report and the integration documents are committed by the owner (`git status` clean on the working branch).
- No `bench.exe`, `whisper-cli.exe`, `tts.exe` or `speechlab.exe` is running (`tasklist`).

## Tasks

1. **Practice-PC probe kit.** Write `scripts/make_probe_kit.ps1` (assembles a git-ignored `probe-kit/` folder: release `bench.exe`, `whisper-cli.exe` with its four DLLs from `src-tauri/binaries/`, the sherpa-onnx models the owner approves to carry, a **small fixed subset** of the dataset audio and metadata of the owner's own voice, and `run-probe.ps1`) and `scripts/practice_pc_probe.ps1` (the file that runs on the practice PC). It must be read-only on the PC apart from its own output folder, must not install anything, and must write an aggregate-only result file (CPU model, cores, RAM, OS, power plan, AC power, `bench preflight` average load, then for each candidate model: RTF median and p95, peak memory, busy cores, WER on the subset with the number of sentences). If `whisper-cli.exe` exits with code 127 the script records "missing runtime library" (this is also the first clean-machine datum for SpeechLab I-070). Candidate models: whisper.cpp tiny, base q5_1, small q5_1 and, only if the owner approves the 574 MB download, large-v3-turbo q5_0; Parakeet only if the owner wants the CC BY 4.0 case measured (decision Q-01).
2. **Doc** `docs/integration/PRACTICE_PC_PROBE.md`: how to build the kit, copy it, run it, bring back the result file, what to do if the PC is busy, what the numbers mean, the proposed budgets of plan section 4. And `docs/integration/PROBE_RESULTS.md`: an empty results template with the same columns as the SpeechLab report tables.
3. **Repair-pair export.** Write `scripts/export_repair_pairs.py` (run with `python -I -X utf8`): from the **public** result folders (`20261007-215116-full-owner-reps1-clean` and the `*-t4-*` runs, never `benchmark/results/private/`) list `heard -> expected` for every key term that was not found (drug names, technical terms, names, places), with the engine and decoding that produced it, deduplicated, plus a "must-not-change" list of common words that appeared unchanged and correct. Output `docs/integration/repair-pairs-candidate.json`. It is a candidate fixture for the KB contract `tests/fixtures/knowledge/repair-fr.json`: say so in the header and in the doc; it covers one speaker and the SpeechLab sentences only. Check the output by reading it; remove anything that is not a public script word.
4. **Decision sheet** `docs/integration/DECISIONS_TO_TAKE.md`: one page, Q-01 to Q-09 with the options, the evidence and the recommendation (from plan section 2), and a column for the owner's answer. No answer is invented.
5. **Verify on this laptop**: build the kit, run the probe on a tiny subset here (quiet machine rule applies), and check the result file is well formed. Record honestly what ran. Do **not** publish speed figures from a disturbed run.
6. **SpeechLab freeze**: add a short status banner at the top of `README.md` and `docs/HANDOFF.md` ("frozen after M8 on 2026-10-09; reopen only to measure; see D-048"); update `docs/HANDOFF.md` sections 3 and 7; log, decisions, issues. Do not change engine code, models or the report's evidence.
7. Give the owner the git block including the **tag** the owner runs (`git tag speechlab-m8-final`, `git push origin speechlab-m8-final`) after the commits are merged. Never run it yourself.

## Out of scope

Any change to AssistantCabinetAI. New measurements for the report. Accent work (dropped, D-046). macOS (on hold, D-048). Choosing the engine (that is the owner's, after the probe).

## Acceptance

The kit builds; the probe script runs here and writes a valid aggregate file; the pair export and the decision sheet exist and
were read by you; SpeechLab docs say "frozen"; `git status` lists no audio, model, kit or private file.

## Final message to the owner (French)

What was done, verified, not verified, limits; how to run the probe on the practice PC in five lines; the decision sheet path;
the git block with the tag; then: "Prochain onglet: Speech 1 — Contracts and decisions (`SP-1-contracts.md`), après avoir répondu au moins à Q-02, Q-03 et Q-04."
