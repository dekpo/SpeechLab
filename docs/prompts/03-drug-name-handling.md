# Prompt 03 — Drug-name and key-term handling (T4)

Paste everything below the line into a new chat session. Prerequisite: the long-audio chunking work
(prompt 02) is committed, and no benchmark is running. No download is needed unless you decide to
test a second model; ask the owner before any download.

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge` or any
   write command on git; you must NOT mention any AI assistant, model, vendor or URL anywhere in
   the repository; document everything in English in `docs/`).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. The last entries of `docs/PROJECT_LOG.md` (the long-audio chunking entry and the clean re-run
   entry), `docs/ISSUES.md` items I-034 and I-036, and `docs/DECISIONS.md` D-030, D-031, D-032 and
   D-035.

## Context
Every engine misspells drug names ("amoxicilline", "ibuprofène" and similar) in the `fr-medical`
category and in the key terms of the dataset (`keyTerms` in `benchmark/samples/*.json`, kind
`drug` or `term`). A missing or wrong drug name is a critical error for the target product (a
medical secretary assistant). The critical-error detector already flags a missing key term. The
goal is to find out which cheap technique reduces these errors, how much, and at what risk, on one
speaker and CPU only.

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe` or
   `whisper-cli.exe` running).
2. **Baseline**: from the clean full run `20261007-215116-full-owner-reps1-clean`, list per
   configuration the key-term errors (drug and term kinds separately): how many key terms occur in
   the dataset, how many are missing or wrong per engine. Use `runs.jsonl`, do not re-run anything.
3. **Technique A, engine-side biasing**:
   (a) sherpa-onnx hotwords with a transducer model (Parakeet TDT v3 int8) and modified beam
   search: check in the `sherpa-onnx` crate (`offline_asr.rs`) and by a small experiment whether
   the NeMo transducer path supports hotwords at all; if it does not, record that as a finding and
   do not force it;
   (b) whisper.cpp initial prompt (`--prompt`, vocabulary list) for tiny, base and small.
   Make the vocabulary a plain text file under `benchmark/`, built ONLY from the key terms of the
   dataset plus a SMALL invented list of common drug names. No real clinical content, no clinical
   advice. Keep the provider interfaces: no engine hard-coded in the UI.
4. **Technique B, post-correction**: a dictionary step that snaps near-miss words to a vocabulary
   (edit distance with a strict threshold and a minimum word length), never touching numbers,
   units, negations, weekdays or months. Put it in a new Rust module in `src-tauri/src/speech/`,
   unit-test it on synthetic strings, and keep it OFF by default.
5. **Experiment** (quiet machine: `bench preflight` first, never `--force`; do not compile or run
   anything else while a benchmark runs): compare baseline, A and B on `fr-medical` and
   `fr-with-english-terms` (and `en-technical` for the English terms), one repetition. For every
   configuration report: WER, key-term hits and misses (drug and term separately), critical flags,
   and the FALSE CORRECTIONS, meaning a correct word turned into a wrong one. A technique that
   fixes three drug names and breaks five other words is a regression: say so. State clearly that
   a dictionary correction can silently replace a real but unusual word, and what that would mean
   for a medical text.
6. **Findings**: which technique helps which engine, by how much, with which risk. One speaker, one
   microphone, a small vocabulary taken from the dataset itself: the result is an upper bound on
   how well a vocabulary that is known in advance can help, not a general claim.
7. **Document** (all English): `docs/PROJECT_LOG.md` (Done / Verified / Failed or surprises / Not
   verified / Next), `docs/DECISIONS.md` (techniques tested, defaults, risks), `docs/ISSUES.md`,
   `README.md`, and sections 3 and 7 of `docs/HANDOFF.md`; update `docs/prompts/README.md` if you
   add a prompt file.
8. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the
   process list recorded by `bench preflight` can contain an editor name, replace it by a neutral
   label in the result folder's `config.json`) and make sure no private content was written.
9. **Final message to the owner, in French**: what was done, verified, not verified, limits, the
   exact git commands for the owner to run (never run them yourself; check `git status` first so
   no path is forgotten), and prepare the prompt file for the following step from the backlog in
   `docs/HANDOFF.md` (private accent clips, T5).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software or download anything without asking. Do not touch the
  owner's other app (port 1420) or their Docker/WSL work.
- Private audio (`wav/`, `benchmark/audio/`, `benchmark/samples-private/`) must never be quoted.
