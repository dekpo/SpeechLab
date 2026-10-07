# Ready-to-paste prompts for the next tasks

Every prompt below starts with the same preamble (P0). Paste P0 first, then the task block (T1 to
T8). Prompt `01-clean-benchmark-rerun.md` is separate because it is the immediate next task.

## P0 — Common preamble (paste before any task)

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Read, in this order: `AGENTS.md` (mandatory: never run `git commit`/`push`/`merge`; never
mention any AI assistant, model, vendor or URL in the repository; document everything in
`docs/`), `docs/HANDOFF.md`, the last entries of `docs/PROJECT_LOG.md`, and the open items of
`docs/ISSUES.md`. Start by reading `git status`/`git log` (read-only) and `tasklist` (no benchmark
must be running). Work in small tested steps, write failures down, mark claims VERIFIED or NOT
VERIFIED, never invent numbers. Benchmarks and any timing figure require a quiet machine: run
`bench preflight` first and never use `--force` for publishable figures. At the end: update
`docs/PROJECT_LOG.md`, `DECISIONS.md`, `ISSUES.md`, `README.md` and sections 3 and 7 of
`docs/HANDOFF.md`; check that no AI or vendor name and no private content is in the repository;
give the owner the exact git commands (checking `git status` so no path is forgotten) without
running them; summarise in French what is done, verified, not verified, limits and next step.

## T1 — (done by prompt 01) clean benchmark re-run

## T2 — Timing study with repetitions (I-012)

Goal: measure run-to-run variance and warm versus cold behaviour on a quiet machine.
Steps: run `bench preflight`; then, from `src-tauri`,
`./target/release/examples/bench.exe run --reps 3 --category fr-general,en-general --label timing-3reps`
(about 24 + 16 sentences x 3 repetitions x 9 configurations; estimate the duration before starting
and tell the owner). Compute per configuration: median, p95, min, max and the coefficient of
variation of inference time across repetitions of the same sample, the cold versus warm difference
for sherpa-onnx (first repetition versus the others), and the stability of transcripts across
repetitions (`unstableSamples`). Report in `docs/PROJECT_LOG.md`; close or update I-012 and I-036.
Do not change the scoring or the code except to add a clearly needed report column.

## T3 — Long-audio chunking with VAD (I-017, I-018, I-019, I-029)

Goal: find out whether splitting long audio at silences removes truncation and repetition loops,
and where the limit of each engine lies.
Steps: sherpa-onnx ships a Silero VAD API (see the `sherpa-onnx` crate `vad.rs`); the VAD model
file must be added to `src-tauri/models-manifest.json` as a downloadable support model with its
licence and SHA-256 (verify the licence is permissive; ask the owner before any large download).
Add an engine-independent chunking step in Rust (segments of at most about 25 s cut on silence,
small padding, never cutting inside speech), usable with every provider, plus a `--chunking vad`
option in `bench`. Test on `fr-dict-01`, `fr-dict-02`, `en-dict-01` and on the owner's long private
clips (private samples only; never quote their content). Compare whole-clip versus chunked output
per engine: WER, output length versus reference, repetition loops, truncation, and time.
Unit-test the chunker on synthetic audio. Report limits per engine honestly.

## T4 — Drug-name and key-term handling

Goal: reduce critical errors on drug names (every engine misspells "amoxicilline", "ibuprofène").
Options to test and compare on the `fr-medical` category and the key terms of the dataset:
(a) sherpa-onnx hotwords with a transducer model (Parakeet) using modified beam search and a
hotwords file; (b) whisper.cpp initial prompt (`--prompt`) containing the vocabulary; (c) a
post-correction step that snaps near-miss words to a dictionary (edit distance with a strict
threshold, never changing numbers, units or negations). Use only a SMALL INVENTED test dictionary
taken from the dataset's key terms; do not import real clinical content or give clinical advice.
Measure WER, critical flags and false corrections (a correct word changed into a wrong one). The
dictionary approach must be reported with its risk of silently correcting a real mistake.

## T5 — Private accent clips (Swiss-Romande, Maghreb accent)

Goal: add long clips from other speakers as PRIVATE samples and evaluate them honestly.
Rules: these are voices of third parties; they stay in `benchmark/audio/` and
`benchmark/samples-private/` (git-ignored), results go to `benchmark/results/private/`, and only
aggregate numbers (never content, names or addresses) may be written in documents. Ask the owner
for each clip: who the speaker is in general terms (for example "female, Swiss-Romande"), whether
the speaker consented, and the source/licence of downloaded material (URL required; without a clear
licence it stays private). The owner supplies the reference text of each clip (verbatim, typed by
the owner). Long clips need the chunking of T3 first, or must be cut by the owner into sentences.
Extend `bench` or add an import command so a sample can be created from an existing WAV and a
reference text without code changes. One speaker never represents an accent: write "this speaker".

## T6 — M6: text-to-speech laboratory (plan section 5C)

Goal: evaluate sherpa-onnx TTS for French and English: voice list with metadata (language, gender
only when documented, licence), playback, pause/resume/stop where supported, speech rate, WAV
export, generation time versus audio duration. Candidates and licence flags are in
`docs/M0_FEASIBILITY.md` section 5: avoid `vits-mms-fra` (non-commercial) and Piper `fr_FR-tom`
(AGPL); the espeak-ng phonemizer (GPL-3.0) in the Piper/Kokoro path is an open licensing risk
(D-012): document the exposure and test a model that does not need it if one exists. Use the
`TextToSpeechProvider` trait already declared in `src-tauri/src/speech/`. Ask the owner before
large downloads. Do not auto-play audio. Naturalness is subjective: ask the owner for listening
notes and record them as opinion, not measurement. TTS output can also become synthetic,
licence-clean test audio for the STT benchmark (record its provenance).

## T7 — M7: packaging validation

Goal: build and run a packaged Windows app (`pnpm tauri build`): where native libraries and
`whisper-cli` go (sidecar), model folder handling, startup time, microphone permission in the
release origin (I-024), antivirus interaction, installer size. macOS cannot be tested here: write
the exact validation steps for a Mac (Apple Silicon and Intel), mark everything NOT VERIFIED, and
list the platform differences (Core ML/Metal, WKWebView audio decoding, notarisation).

## T8 — M8: final report and licensing table

Goal: write `docs/SPEECH_ENGINE_EVALUATION.md` with the sections required by `Plan.md` section 12,
using only measured evidence from `benchmark/results/` (cite the run folders), the licensing table
(software, models, runtimes, phonemizers, datasets: licence, commercial use, redistribution,
attribution; flag NeMo CC-BY-4.0 attribution and espeak-ng GPL-3.0; this is research, not legal
advice), the Windows/macOS compatibility statement (verified versus not verified), privacy and
offline validation (including a real test with the network disabled), and an architecture
recommendation that does not force a winner when the evidence is inconclusive. Include the
roadmap for integrating into AssistantCabinetAI.
