# Prompt 04 — Private accent clips (T5)

Paste everything below the line into a new chat session. Prerequisite: the drug-name work
(prompt 03) is committed, and no benchmark is running. This task needs the owner's input (the clips
and their typed reference texts): the agent must ask for them and must NOT invent or guess any
reference text. No download is needed unless the owner agrees to one.

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge` or any
   write command on git; you must NOT mention any AI assistant, model, vendor or URL anywhere in
   the repository; document everything in English in `docs/`).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. The last entries of `docs/PROJECT_LOG.md` (the drug-name entry, the long-audio chunking entry),
   `docs/DECISIONS.md` D-015 (consent and licence rule for third-party voices), D-035 and D-036,
   and `docs/prompts/next-tasks.md` section T5.

## Context
Every measurement so far uses ONE speaker (the owner, native French, no marked accent) and one
microphone. The plan wants accents (Swiss-Romande, Maghreb) because the target users speak with
them. The owner has recordings of other people; those are PRIVATE: a voice is personal data. They
must stay out of the repository (`benchmark/audio/` and `benchmark/samples-private/` are
git-ignored, results of private runs go to `benchmark/results/private/`), and no document may
contain their content, names, addresses or any quoted text: only aggregate numbers (WER, counts,
timings) and a general description of the speaker such as "female, Swiss-Romande, this speaker".
One speaker never represents an accent: write "this speaker", never "the Swiss accent".

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe` or
   `whisper-cli.exe` running). Read how private samples are handled today: `src-tauri/src/speech/dataset.rs`
   (`samples-private`, `private` flag, `--include-private`), `benchmark/README.md`.
2. **Ask the owner** (in French) for, per clip: a file path, a general description of the speaker
   (gender only if the owner states it, accent, language), whether the speaker consented, and the
   verbatim reference text typed by the owner. For downloaded material a source URL and a licence are
   required; without a clear licence the clip stays private. Do not listen to, transcribe or "fix"
   the reference yourself: the reference is the owner's ground truth. If a clip is long (over 25 s),
   use `--chunking vad` (D-035) and say so; Canary and sherpa Whisper tiny are unreliable on long
   clips, so judge them on short pieces.
3. **Import without code changes per clip**: add (or extend) a small `bench` subcommand, for example
   `bench import --wav <file> --id <id> --reference-file <txt> --speaker <id> --accent <text> --private`,
   that converts the audio to 16 kHz mono 16-bit WAV (reuse `wav.rs`), copies it to
   `benchmark/audio/`, writes the metadata JSON to `benchmark/samples-private/`, and validates it with
   the existing dataset checks. Unit-test it on synthetic audio. The reference file is read, never
   printed.
4. **Run** on a quiet machine (`bench preflight` first, never `--force`, nothing else running):
   `bench run --include-private --category <accent category> --label <name>` for the useful
   configurations (at least Parakeet, Canary, whisper.cpp small with 5 beams; add the others if time
   allows). One repetition is enough for accuracy (the engines are deterministic, four runs agreed).
5. **Report only aggregates**: WER with its interval (`scripts/bootstrap_ci.py` works per sentence;
   with few clips say that the statistical power is very low), critical flags as COUNTS, speed.
   Compare with the owner's own recordings of comparable length, and say that the speaker, the
   microphone and the room differ at the same time, so the cause of a difference (accent, device,
   noise) cannot be separated.
6. **Optional, only if the owner asks**: try the key-term vocabulary of D-036 on the accent clips to
   see whether it helps the same way (a vocabulary known in advance is an upper bound).
7. **Document** (all English, no private content): `docs/PROJECT_LOG.md` (Done / Verified / Failed or
   surprises / Not verified / Next), `docs/DECISIONS.md`, `docs/ISSUES.md`, `README.md`, sections 3
   and 7 of `docs/HANDOFF.md`; update `docs/prompts/README.md` if you add a prompt file.
8. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the
   process list recorded by `bench preflight` can contain an editor name, replace it by a neutral label
   in the result folder's `config.json`), and make sure `git status` shows nothing under
   `benchmark/audio/`, `benchmark/samples-private/` or `benchmark/results/private/`.
9. **Final message to the owner, in French**: what was done, verified, not verified, limits, the
   exact git commands for the owner to run (never run them yourself; check `git status` first so no
   path is forgotten), and prepare the prompt file for the following step from the backlog in
   `docs/HANDOFF.md` (M6, the text-to-speech laboratory).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software or download anything without asking. Do not touch the
  owner's other app (port 1420) or their Docker/WSL work.
- Private audio and private results must never be quoted, committed or pasted into a document.
