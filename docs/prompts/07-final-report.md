# Prompt 07 — M8: final report, licensing table, recommendation

Paste everything below the line into a new chat session. Prerequisites: the M7 work (packaging validation,
`docs/MACOS_VALIDATION.md`, decisions from D-044 on) is committed and no benchmark is running. This task is
writing and cross-checking, not measuring: it needs no new recording, no new model and no listening. The accent
evaluation was dropped by the owner (D-046): the report states that no accent figure exists and that accents
are out of scope.

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge`, `git rebase`,
   `git tag` or any write command on git; you must NOT mention any AI assistant, model, vendor or URL
   anywhere in the repository; document everything in English in `docs/`; do not install software or
   change the system without the owner's approval).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. `Plan.md` section 12 (the exact list of sections the report must contain) and section 4 and 9
   (requirements, packaging).
4. The whole of `docs/PROJECT_LOG.md` (it is the evidence trail), `docs/DECISIONS.md` (all entries),
   `docs/ISSUES.md` (all open items), `docs/TTS_LICENSES.md`, `docs/TTS_LISTENING_NOTES.md`,
   `docs/MACOS_VALIDATION.md`, `docs/M0_FEASIBILITY.md`, `benchmark/README.md`.
5. The result folders in `benchmark/results/` (never `benchmark/results/private/` contents: aggregate numbers
   only, see HANDOFF section 2 rule 6 and I-047).

## Task
Write `docs/SPEECH_ENGINE_EVALUATION.md` with the sections of `Plan.md` section 12, using only measured
evidence:

1. **Executive summary** — findings and unresolved questions, half a page.
2. **Engine comparison** — whisper.cpp versus sherpa-onnx, each figure with its run folder, decoding mode,
   repetition count and recorded background CPU load (D-018, D-034). Quote speed with the 10 to 20 % margin
   of the timing study. State the single-speaker limit.
3. **French accuracy, English accuracy** — WER, CER, critical errors, bootstrap intervals
   (`scripts/bootstrap_ci.py`), with the confidence caveats. Accents: say plainly that no accent figure exists
   and that the owner dropped that evaluation (D-046).
4. **Specialised vocabulary** — drug names, key terms, what helped and what regressed (D-036).
5. **TTS assessment** — voices, languages, speed control, latency (real-time factor), the owner's listening
   results (as the owner's judgement, quoted from `docs/TTS_LISTENING_NOTES.md`), the read-along and the
   normaliser (D-042, D-043).
6. **Performance** — measured CPU, RAM, start-up time (the packaged-app measurements of M7), processing speed.
7. **Windows/macOS compatibility** — verified on Windows (installer, sidecar, offline proof, microphone in the
   release origin) versus macOS NOT VERIFIED (link `docs/MACOS_VALIDATION.md`).
8. **Privacy and offline operation** — what was proven (M7 offline proof) and what was not.
9. **Licensing** — a table with one row per piece of software, runtime, phonemizer, model, voice and dataset:
   licence, commercial use, redistribution, attribution, source of the information, rating. Flag
   explicitly: espeak-ng GPL-3.0 inside every TTS-capable build (D-012, I-051, I-054), NeMo-derived models
   CC BY 4.0 attribution, the Piper siwis and gilles restricted parent checkpoints (D-043, I-063: re-train from
   the clean base before any commercial release), the `excluded` and `review` voices, models never
   bundled. Mark it as research, not legal advice, and list what a lawyer must read.
10. **Architecture recommendation** — one of the options of Plan.md section 12, or a combination, justified by
    the evidence. **Do not force a winner when the evidence is inconclusive**: say so and list what would settle it.
11. **Integration roadmap** — a sequence of steps for AssistantCabinetAI, each with its precondition (licence
    decision, French voice re-training, signing certificate, macOS validation, larger
    speaker panel).
12. **Appendix: claims table** — every claim in the report marked VERIFIED (with run folder or log entry) or
    NOT VERIFIED (with the exact steps), as `AGENTS.md` section 3 requires.

## Method
- Build the report from the project's documents, then **cross-check each number against its source**
  (`summary.md`, `summary.csv`, `system.json`, the log entry). Fix the documents if you find a contradiction,
  and record it in `docs/PROJECT_LOG.md`. Read generated tables before trusting them.
- Do not run or invent measurements. If a figure is missing, say it is missing. If you need a re-score,
  `bench rescore` and `bench summarize` work offline on stored transcripts.
- No private content: aggregate numbers only for the owner's private clips; no first names; no quotes from
  `benchmark/results/private/`.
- No simulated or invented benchmark data; no AI or vendor names (third-party project names appear only as
  sources and licensors).

## Documentation (all English)
`docs/PROJECT_LOG.md` entry (Done / Verified / Failed or surprises / Not verified / Next), `docs/DECISIONS.md`
(the recommendation as a decision, with the alternatives), `docs/ISSUES.md` (new or closed items),
`README.md` (status, link to the report), sections 3 and 7 of `docs/HANDOFF.md`, `docs/prompts/README.md`
(mark 06 and 07 done). Five headings at the end of the milestone: **Implemented / Verified / Not verified /
Known limitations / Next step**.

## Identity and privacy check
Search the repository for AI or vendor names (none allowed), for private content, and make sure `git status`
lists no installer, no `target/` output, no `src-tauri/binaries/`, no model or audio file.

## Final message to the owner, in French
What was done, verified, not verified, limits, the recommendation in five lines, the exact git commands for the
owner to run (never run them yourself; check `git status` first so no path is forgotten; one English sentence
per commit message, imperative, about 72 characters, no trailer), and the proposed next step after the report
(re-training the French voices, a Mac for `docs/MACOS_VALIDATION.md`, code signing).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software or download anything without asking. Do not touch the owner's other
  app (port 1420) or their Docker/WSL work.
- Very long inline shell or Python scripts with apostrophes were rejected by the agent shell: write script
  files and run them. `python -I` ignores `PYTHONIOENCODING`: use `python -I -X utf8`.
