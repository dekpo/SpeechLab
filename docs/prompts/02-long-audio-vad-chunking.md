# Prompt 02 — Long audio: VAD chunking

Paste everything below the line into a new chat session. Prerequisite: the timing study (T2) is
merged or at least committed, and no benchmark is running. The owner should agree to a model-sized
download (the voice-activity-detection model is small, about 2 MB, but ask first).

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge` or any
   write command on git; you must NOT mention any AI assistant, model, vendor or URL anywhere in
   the repository; document everything in English in `docs/`).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. The last entries of `docs/PROJECT_LOG.md` (the timing-study entry and the clean re-run entry),
   `docs/ISSUES.md` items I-017, I-018, I-019, I-029, I-035, and `docs/DECISIONS.md` D-032.

## Context
Short sentences are handled well, but long audio is not: Canary emits a warning on longer audio
and can drop the ending (I-017, I-019), sherpa-onnx Whisper tiny falls into repetition loops
(I-018), the behaviour on long clips is unknown (I-029), and Canary once produced runaway garbage
(I-035). The idea to test: cut long audio at silences into pieces of at most about 25 s, transcribe
each piece, join the texts. This must be engine-independent so it works with every provider.

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe` or
   `whisper-cli.exe` running).
2. **Voice-activity-detection model**: the `sherpa-onnx` crate has a Silero VAD API (see its
   `vad.rs`). Add the VAD model to `src-tauri/models-manifest.json` as a downloadable SUPPORT model
   (not an STT model: check how `models.rs` and the UI list models, and keep it out of the engine
   pickers) with its licence and SHA-256. Verify that the licence is permissive and record it in
   `docs/DECISIONS.md`. Ask the owner before downloading; propose the exact command, do not install
   anything silently.
3. **Chunker in Rust** (new module in `src-tauri/src/speech/`, provider-independent): input =
   mono 16 kHz samples; output = a list of segments of at most about 25 s, cut on silence, with a
   small padding, never cutting inside speech when a pause exists; a fallback cut at the quietest
   point when a stretch of speech is longer than the limit. Unit-test it on synthetic audio
   (generated tones and silence, no real recordings in tests): short clip stays one segment, long
   clip with pauses is split at the pauses, continuous speech is cut at the quietest point, empty
   and all-silence input.
4. **Use it**: add `--chunking vad` (default off) to `bench` and keep the whole-clip path unchanged
   so the old results stay comparable. Record in `runs.jsonl` whether chunking was used and the
   number of segments. Sum the inference times of the segments. Do not change the scoring.
5. **Experiment** (quiet machine: `bench preflight` first, never `--force`; do not compile or run
   anything else while a benchmark runs): compare whole-clip versus chunked output on the long
   samples, `fr-dict-01`, `fr-dict-02`, `en-dict-01` (they are in the committed dataset), and, as
   PRIVATE samples only, the owner's longer private clips if present (never quote their content,
   only aggregate numbers). For every configuration report: WER, output length versus reference
   length, repetition loops (runaway outputs), truncation (missing ending), and time. Also run the
   short categories once with chunking on to prove that short audio is not harmed (the output must
   be identical when there is one segment).
6. **Findings**: say honestly where each engine's limit lies (maximum clip length it handles
   without chunking, what chunking fixes, what it does not fix, for example the repetition loops of
   Whisper tiny inside a single segment). One speaker, one microphone: no generalisation.
7. **Document** (all English): `docs/PROJECT_LOG.md` (Done / Verified / Failed or surprises / Not
   verified / Next), `docs/DECISIONS.md` (chunker design, VAD model licence, defaults), update
   I-017, I-018, I-019, I-029, I-035, `README.md`, and sections 3 and 7 of `docs/HANDOFF.md`;
   regenerate the prompt list in `docs/prompts/README.md` if you add a prompt file.
8. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the
   process list recorded by `bench preflight` can contain an editor name, replace it by a neutral
   label in the result folder's `config.json`) and make sure no private content was written.
9. **Final message to the owner, in French**: what was done, verified, not verified, limits, the
   exact git commands for the owner to run (never run them yourself; check `git status` first so no
   path is forgotten), and prepare the prompt file for the following step from the backlog in
   `docs/HANDOFF.md` (drug-name handling, T4).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software without asking. Do not touch the owner's other app
  (port 1420) or their Docker/WSL work.
- Private audio (`wav/`, `benchmark/audio/`, `benchmark/samples-private/`) must never be quoted.
