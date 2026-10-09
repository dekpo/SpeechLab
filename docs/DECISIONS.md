# Decisions

One entry per non-trivial choice. Never delete a superseded decision; mark it `Superseded by D-xxx`.

---

## D-001 — Tauri 2.x stable, not Tauri 3 alpha
- Date: 2026-10-06 · Status: accepted
- Context: Tauri 2.12.1 is stable; 3.0.0-alpha.4 exists. The plan specifies Tauri 2.
- Decision: pin Tauri 2.x. Ignore 3.x alpha.
- Consequences: re-evaluate only when 3.x is stable and the owner asks.

## D-002 — Start with sherpa-onnx; whisper.cpp afterwards
- Date: 2026-10-06 · Status: accepted (owner decision)
- Context: `Plan.md` suggests M2 = whisper.cpp, M3 = sherpa-onnx. sherpa-onnx has prebuilt libraries and an official Rust crate (no CMake/LLVM needed), and can also run Whisper models.
- Decision: swap the order: **M2 = sherpa-onnx STT, M3 = whisper.cpp**. Milestone content is unchanged; this is a re-ordering, not a skip.
- Consequences: CMake and LLVM are not needed until M3. The provider contract is designed in M1 so both orders work.

## D-003 — Free and open-source first
- Date: 2026-10-06 · Status: accepted (owner decision)
- Decision: prefer free/open-source tools, libraries, and models. Any exception must be listed here with justification.
- Known exception candidate: **MSVC Build Tools** (required by Tauri on Windows) is free to download but is not open source and has its own license terms. Alternative toolchains are not officially supported by Tauri. Owner to confirm acceptance.
- Free/open tools in use: Node.js (MIT), Rust (MIT/Apache-2.0), Tauri (MIT/Apache-2.0), React (MIT), Vite (MIT), pnpm (MIT), Git (GPL-2.0, as a tool only), CMake (BSD-3) and LLVM (Apache-2.0 with LLVM exception) when needed.

## D-004 — espeak-ng (GPL-3.0) is a flagged licensing risk
- Date: 2026-10-06 · Status: open
- Context: sherpa-onnx's Piper/VITS/Kokoro TTS path builds against an espeak-ng fork for phonemization. Upstream espeak-ng is GPL-3.0.
- Decision: do not hide the issue. Continue TTS experiments (research only) but record the risk; seek legal review before any commercial use. Evaluate TTS models that do not need espeak-ng (e.g. char-based VITS, other architectures).
- Consequences: Piper/Kokoro may be unusable commercially even though their weights are permissively licensed.

## D-005 — Model license exclusions
- Date: 2026-10-06 · Status: accepted
- Excluded for the commercial objective: `vits-mms-fra` (CC-BY-NC-4.0), Piper `fr_FR-tom` (AGPLv3). CC-BY-SA (Piper upmc) requires review. CC-BY models are acceptable with attribution.

## D-006 — Architecture: provider traits behind a Tauri command contract
- Date: 2026-10-06 · Status: accepted (details in `docs/M0_FEASIBILITY.md` §8)
- Decision: Rust traits `SpeechToTextProvider` / `TextToSpeechProvider`, adapters per engine, UI only sees an engine-agnostic JSON contract. Models downloaded on demand, never bundled.

## D-007 — Audio capture strategy
- Date: 2026-10-06 · Status: proposed (to verify in M4)
- Decision: try WebView `getUserMedia` first (least native code); fall back to `cpal` if microphone permission or quality is a problem.

## D-008 — Dev server port 1430 (not Tauri's default 1420)
- Date: 2026-10-06 · Status: accepted
- Context: port 1420 is used by the AssistantCabinetAI desktop dev server on the owner's machine. SpeechLab must stay independent and must not stop or reuse it.
- Decision: Vite `server.port = 1430` with `strictPort`, and `devUrl` set to the same value in `src-tauri/tauri.conf.json`. Both must be changed together.

## D-009 — Mock provider is allowed only as a clearly flagged M1 scaffold
- Date: 2026-10-06 · Status: accepted
- Context: M1 needs a round trip before any engine exists, but `Plan.md` forbids simulated benchmark data.
- Decision: the mock sets `is_mock = true`, the UI shows a warning banner, and the benchmark service (M5) must refuse results from mock providers. The mock is removed from the default registry once a real provider exists, or kept only for tests.

## D-010 — Hand-written scaffold instead of `create-tauri-app`
- Date: 2026-10-06 · Status: accepted
- Decision: keep the project minimal and understandable; no template-specific files. Bundling is disabled until M7. The Rust speech contract lives in `src-tauri/src/speech/` for now and can become a separate crate when integrated into AssistantCabinetAI.

## D-011 — Placeholder icon
- Date: 2026-10-06 · Status: accepted
- Decision: a plain blue square (`app-icon-source.png`) generated locally, used only because Tauri requires icons. Replace before any distribution.

## D-012 — sherpa-onnx linking and the espeak-ng (GPL-3.0) exposure
- Date: 2026-10-06 · Status: open (extends D-004)
- Evidence: the crate's default static link includes espeak-ng; the STT-only executable contains it (65 "espeak" matches). A build against the official `no-tts` libs has none and gives identical STT output.
- Decision for now: keep the default libs during M2 development (simplest), keep the no-tts build as the candidate for any distributable STT-only binary, and decide the TTS strategy in M6 (TTS needs the full libs; options: separate TTS process/binary, non-espeak voices, or legal clearance). Not legal advice: whether static linking triggers GPL obligations here is a question for counsel.

## D-013 — Models stored outside the repo, in the app data directory
- Date: 2026-10-06 · Status: accepted
- Decision: `<app data dir>/models` (Windows: `%APPDATA%\ai.assistantcabinet.speechlab\models`), overridable with `SPEECHLAB_MODELS_DIR`. Never committed, never bundled. Each model dir has a marker file written only after the checksum and file layout were verified.

## D-014 — Download stack uses the OS certificate store
- Date: 2026-10-06 · Status: accepted
- Context: TLS-intercepting antivirus or proxies break rustls with bundled roots.
- Decision: `ureq` with the `native-certs` feature for model downloads. We do not disable the antivirus or TLS verification. The crate build-script download cannot be changed, hence `scripts/fetch-sherpa-libs.ps1`.

## D-015 — Model checksums are pinned in the manifest
- Date: 2026-10-06 · Status: accepted
- Decision: use the digest published by the official release when available; otherwise trust-on-first-use (hash computed locally on first download, then pinned) and say so (Whisper tiny is in that case). A mismatch deletes the file and fails the install.

## D-017 — whisper.cpp runs as an external process (`whisper-cli`), not via `whisper-rs`
- Date: 2026-10-06 · Status: accepted (revisit after M5 measurements)
- Context: `whisper-rs` needs CMake plus libclang (bindgen), and its bundled whisper.cpp version was unverified.
- Decision: build whisper.cpp from the official tag with CMake only and call `whisper-cli` as a child process. Benefits: no LLVM, crash isolation, real cancellation by killing the process, MIT binary kept separate from the app. Costs: model reloaded every run (cold start each time), process-spawn overhead, a second binary to ship (Tauri sidecar in M7).
- Consequences: LLVM is not installed. A persistent server or in-process binding can be reconsidered if cold-start cost matters for the product.

## D-018 — Decoding strategy is reported, never hidden
- Date: 2026-10-06 · Status: accepted
- Context: `whisper-cli` defaults to 5-beam search; sherpa-onnx Whisper uses greedy search. Comparing them naively would confound engine and decoding.
- Decision: every result carries `decoding`; whisper.cpp beam size is configurable (`SPEECHLAB_WHISPER_BEAM_SIZE`, default 5 = upstream default). M5 benchmarks must run both beam 5 and greedy (beam 1).

## D-019 — whisper.cpp built with `NMake Makefiles`
- Date: 2026-10-06 · Status: accepted
- Context: the Visual Studio CMake generator relies on `vswhere.exe`, which is missing here (I-022).
- Decision: build inside the MSVC environment (`vcvars64.bat`) with NMake. Works, single-threaded build (91 s). Ninja would be faster but is one more tool to install.

## D-020 — Microphone capture in the WebView (confirms D-007)
- Date: 2026-10-07 · Status: accepted
- Decision: getUserMedia + AudioWorklet, processing flags off, 16 kHz mono 16-bit WAV, saved through Rust. Works on Windows/WebView2 here. Costs: a native permission prompt, and macOS behaviour unverified. `cpal` stays the fallback if the prompt or quality becomes a problem.

## D-021 — Non-WAV import decoded by the WebView, not by a Rust decoder
- Date: 2026-10-07 · Status: accepted (revisit for macOS)
- Context: voice notes are usually Ogg/Opus or M4A. A Rust decoder (symphonia, MPL-2.0) handles MP3/AAC/Vorbis/FLAC but not Opus.
- Decision: use `decodeAudioData` in the WebView and resample to 16 kHz mono, so no extra dependency. The Rust-side magic-byte check still names unsupported formats when a path is typed by hand. Support depends on the platform WebView; macOS WKWebView does not reliably decode Ogg/Opus (NOT VERIFIED), so an alternative must be chosen before shipping on macOS.

## D-022 — Basic text normalisation and what it must not hide
- Date: 2026-10-07 · Status: accepted (extended in M5)
- Decision: scoring normalisation = lower-case, punctuation to spaces, hyphen/dash to space, apostrophes unified. Kept: digits, decimal separators between digits, `%`. Not done: number-word equivalence ("trois" vs "3"), Unicode NFC. Originals are always preserved. Critical semantic errors (numbers, units, dosages, negations, drug names) get their own detector in M5; WER alone is never a safety statement.

## D-022 update (2026-10-07) — Spoken numbers and unit spellings are now folded
- Scoring folds spoken numbers to digits on both sides and unifies listed unit spellings, so "quinze" = "15" and "milligrammes" = "mg" no longer inflate WER. Different values ("15" vs "50") and different units ("mg" vs "g") remain errors. Swiss number words (septante, huitante, nonante) are not covered yet (I-030). Implementation: `numbers.rs`; the critical-error detector (`critical.rs`) runs on the same folded tokens.

## D-026 — Critical errors are flags, separate from WER
- Date: 2026-10-07 · Status: accepted
- Decision: a transcript is reported with BOTH its WER/CER and a list of critical flags (numbers, units, negations, weekday/month, missing key terms). A run with critical flags is never presented as "good" because its WER is low. Heuristic detector: every flag keeps expected/found text for human review, and false alarms/misses will be counted on real outputs in M5c.

## D-027 — Private samples live in a git-ignored folder
- Date: 2026-10-07 · Status: accepted
- Decision: `benchmark/samples-private/` (metadata and references) and `benchmark/audio/` are git-ignored. The loader refuses `third-party-private` samples outside `samples-private/` and refuses to mark them committable. Results computed from private samples go to `benchmark/results/private/` (ignored) or are reported only as aggregate numbers.

## D-028 — The reference transcript always comes from the script
- Date: 2026-10-07 · Status: accepted
- Decision: the recorder stores the script sentence as the reference; the speaker cannot edit it. If a sentence was misread, it is recorded again. This avoids hand-typed references that would silently differ from what was said, and keeps references identical for every speaker. Reading order follows the owner's priorities (D-025).

## D-029 — Benchmark runner design (M5c)
- Date: 2026-10-07 · Status: accepted
- One CLI (`cargo run --release --example bench -- check | run`), library code in `speech/benchmark.rs` so a UI can reuse it later.
- **One child process per model configuration** (default): cold-start time and memory belong to that model alone, and a crash cannot corrupt the other results. `--no-isolate` exists but mixes memory figures.
- **Accuracy from repetition 1; timing from all repetitions.** Output is checked for stability across repetitions (`unstableSamples`).
- **Both decodings for whisper.cpp** (5 beams = upstream default, and 1 = greedy), sherpa-onnx always greedy (D-018).
- **Memory and CPU are sampled** (every ~25 ms resident set; accumulated CPU time), so peaks are lower bounds. For whisper.cpp the child process is probed; for sherpa-onnx the whole benchmark child process (model, runtime and buffers). "Busy cores" = CPU time / (load + inference time); it can exceed the configured thread count when a runtime spins extra threads.
- **Failures are data**: a failed run keeps its error and no metrics; the failure rate is in the summary. Nothing is simulated or imputed.
- **Outputs** (`benchmark/results/<UTC>-<label>/`): `runs.jsonl` (every run), `summary.csv`, `summary.md`, `system.json` (CPU, cores, RAM, OS, build profile, accelerator), `config.json`. Private samples never go to the committed folder (`results/private/`, git-ignored).
- **Release build for speed figures.** Rust code is thin around native engines, but debug builds still add overhead in WAV decoding and bookkeeping; speed figures must come from `--release`.

## D-022 update 2 (2026-10-07) — Number grammar and digit canonicalisation, found on real output
- The first real transcripts showed two problems in my own scoring code: "ten thirty" was added up to 40 (a time is two numbers), and ASR formats "1030" and "12,000" were counted as errors against "10 30" and "12000". Fixes: spoken numbers follow a grammar (a number only extends when the next word is a valid continuation), digit groups are canonical ("12,000" = "12.000" = "12000", "2,5" = "2.5"), and adjacent digit tokens are joined ("10 30" = "1030"). A lost decimal point ("2,5" vs "25") and "2,5" vs "2,500" remain errors. 73 unit tests cover this. Lesson: normalisation rules must be validated on real ASR output before any number is published.

## D-023 — Clips stay local, are listed at startup and deletable
- Date: 2026-10-07 · Status: accepted
- Decision: recordings and converted imports live in `<app data>/recordings` as WAV, never sent anywhere, always visible and deletable in the UI. File access commands refuse any path outside that directory.

## D-024 — Comparison runs are sequential and never ranked
- Date: 2026-10-07 · Status: accepted
- Decision: models run one after another to avoid CPU contention. The UI shows facts (times, ×fastest, WER/CER or disagreement with a baseline) and a standing warning, but no "winner" label, per `Plan.md`.

## D-025 — M5 dataset scope (owner answers, 2026-10-07)
- Date: 2026-10-07 · Status: accepted (languages beyond fr/en are proposed, not decided)
- Languages: French and English as in `Plan.md`. "Technical with English words" means FRENCH sentences containing English terms (PostgreSQL, API REST, Kubernetes...), separate from the English-language set. German and Italian (Swiss context) are outside the plan; candidate optional extension after M5, preferably from public-licence corpora rather than non-native reading.
- Utterance types wanted by the owner: short questions and short statements (voice queries inside AssistantCabinetAI). Long dictation is secondary (reports) and depends on chunking (I-029): engines are measured with and without segmentation.
- Speakers available: owner (standard metropolitan French, no marked accent, own voice, consent implicit); two Swiss Romande female clips (one already used in earlier tests); one long clip with a Maghreb-accented French speaker (not yet downloaded). One speaker per accent can never represent an accent: results are reported as "this speaker", never as "Swiss French" in general.
- Consent and licence rule: recordings of third parties (voice notes) are treated as private local test material: never committed, never quoted, no content in docs, only aggregate numbers reported, and ideally the speaker's consent is obtained. Downloaded clips need a source URL and licence recorded in the dataset metadata, otherwise they stay local-only.

## D-016 — Cancellation contract
- Date: 2026-10-06 · Status: accepted
- Decision: providers declare `supportsCancellation`. sherpa-onnx = false (blocking decode, checked only before start). Downloads are cancellable. Update from M3: whisper.cpp = true (child process killed on cancel, verified by a unit test and from the UI).


## D-030 — Scoring is versioned and results can be re-scored
- Date: 2026-10-07 · Status: accepted
- Context: the first real transcripts exposed five defects in my scoring rules (see PROJECT_LOG M5c). Fixing rules after results exist is legitimate only if it is transparent.
- Decision: every run record carries `scoringVersion`; `bench rescore --dir <folder>` recomputes WER/CER/flags from the stored transcripts with the current rules, keeps the previous file as `runs.scoring-v<N>.jsonl`, and rebuilds the summaries. Rules change only to remove formatting artefacts (never to favour an engine), each change gets a unit test, and the old numbers stay on disk.

## D-031 — Statistical reporting
- Date: 2026-10-07 · Status: accepted
- Decision: WER is always shown with its sampling interval (`scripts/bootstrap_ci.py`, paired bootstrap over sentences) and engines are only called different when the paired interval excludes 0. Suspect samples (most engines disagree with the reference) are listed and can be excluded in a second table. Everything is stated for the recorded speaker only.

## D-032 — Provisional engine shortlist for AssistantCabinetAI (NOT final; M8 decides)
- Date: 2026-10-07 · Status: provisional
- Evidence so far (one speaker, CPU only; speed from the clean re-run at about 20 % background CPU, updated 2026-10-07, the shortlist itself does not change): sherpa-onnx Parakeet TDT v3 int8 is the most accurate and fast (RTF 0.11, inference median 0.55 s), at the cost of 1.9 GB peak memory and 2.0 s cold load; Canary int8 is comparable on clean inputs (RTF 0.11, 1.1 GB) but showed a runaway failure; whisper.cpp small is accurate (3.9 %) but still slower than real time here (RTF 1.12 to 1.24; the disturbed first run had given 1.56 to 1.85); tiny/base Whisper are too inaccurate for French medical or administrative vocabulary.
- Not decided: licensing review of NeMo models (CC-BY-4.0, attribution) and of the espeak-ng exposure (D-012) for the sherpa-onnx route; drug-name handling; long-audio chunking; other speakers and accents.

## D-033 — Benchmarks refuse to run on a busy machine
- Date: 2026-10-07 · Status: accepted
- Decision: `bench run` checks CPU use (limit 15 % average over 8 s) and the power source before starting and aborts with the list of busiest processes, unless `--force`. The check result is stored in `config.json` and the power plan/AC state in `system.json`, so every published speed figure carries the conditions it was measured under. Speed numbers from a forced or disturbed run must be labelled as such and never mixed with clean ones.

## D-034 — Quiet-machine limit relaxed from 15 % to 30 % background CPU (realistic load)
- Date: 2026-10-07 · Status: accepted (owner's decision) · Amends D-033
- Context: on the development laptop the background load never fell below 16 % even with the owner's applications closed and Docker/WSL stopped (10 preflight runs: 16 to 31 %). The load comes from Windows services (firewall filtering engine, DNS and capability services) and from the security software, which cannot be switched off reliably. A client machine running a desktop application will also carry background load, so a speed figure measured on a perfectly idle machine would be optimistic.
- Options: (a) keep 15 % and never run; (b) `--force` (rejected: the run would be labelled disturbed); (c) raise the limit and record the actual load; (d) cut the network first.
- Decision: the limit is 30 % average CPU over 8 s (constant `MAX_BACKGROUND_CPU_PERCENT` in `examples/bench.rs`). Battery is still refused. The measured load is stored in `config.json` (`preflight.idleCpuPercent`) and must be quoted with every speed figure. The run is described as "measured under a realistic background load of about N % CPU", NOT as "idle machine".
- Consequences: speed figures carry more noise than on an idle machine and are not comparable to figures from another machine or load. The 3-repetition timing study (I-012) is needed to quantify the spread. Accuracy is unaffected (deterministic engines, to be confirmed by the comparison with the first run).

## D-035 — Engine-independent chunking of long audio with a voice-activity detector
- Date: 2026-10-08 · Status: accepted (experimental, default OFF)
- Context: long recordings are handled badly by some engines: Canary drops the ending and once emitted runaway text (I-017, I-019, I-035), sherpa-onnx Whisper tiny loops (I-018) and, by its own message, processes only the first 30 s of a longer clip (measured, see the project log), the behaviour on long clips was otherwise unknown (I-029). The dictation use case (reports) needs clips of 30 s to several minutes.
- Options: (a) rely on each engine's own long-form handling (whisper.cpp already windows the audio internally; the others do not); (b) cut at fixed intervals (can cut inside a word); (c) cut at silences found by a voice-activity detector (VAD) and join the texts; (d) streaming recognition (different models, out of scope).
- Decision: (c), implemented ABOVE the provider interface so that it works with every engine and needs no engine change. `src-tauri/src/speech/chunking.rs` (planner, energy detector used by the tests, `transcribe_chunked`) and `vad.rs` (Silero VAD through the `sherpa-onnx` crate).
  - A clip of at most 25 s is NEVER touched: the provider receives the original file, the detector does not run, the output is identical to the whole-clip path (unit-tested; also checked on the benchmark, see the log).
  - A longer clip is cut into segments of at most 25 s, padding included. Speech spans (from the VAD: threshold 0.5, a pause of at least 0.4 s ends a span, bursts under 0.25 s ignored, no VAD-side maximum) are grouped greedily; each cut falls in the middle of the pause between two spans, with 0.2 s of padding kept on both sides of the speech (never past the middle of a short pause, so pieces never overlap and no audio is duplicated). A span longer than a segment may be is cut at the quietest 20 ms-frame run of the second half of the allowed length (fallback, only for continuous speech). Silence-only audio yields no segment and an empty text.
  - Each piece is written as a temporary 16-bit 16 kHz mono WAV and sent through the normal `transcribe` call (so whisper.cpp keeps its own process and cancellation, sherpa-onnx its loaded model); clips that are not at 16 kHz are resampled by linear interpolation first. Texts are joined with single spaces. Inference, load and CPU times are summed over the pieces, peak memory is the maximum; the VAD time is reported separately (`chunkingMs`) and is not part of `inferenceMs`; the real-time factor refers to the whole clip.
  - Default OFF in the benchmark: `bench run --chunking vad` turns it on, and the old whole-clip path is unchanged so earlier results stay comparable. `runs.jsonl` records `chunking` ("off"/"vad"), `segments` and `chunkingMs` (old files read as "off", 1 segment, 0).
- VAD model: Silero VAD, file `silero_vad.onnx` (643 854 bytes, SHA-256 `9e2449e1087496d8d4caba907f23e0bd3f78d91fa552479bb9c23ac09cbb1fd6`), downloaded from the official sherpa-onnx `asr-models` release (about 0.6 MB, not 2 MB as first guessed) and installed by the normal model installer (`transcribe install silero-vad`). Licence: MIT, Copyright (c) 2020-present Silero Team (VERIFIED by reading the LICENSE file of the upstream repository on 2026-10-08): permissive, commercial use allowed, attribution required. It is a SUPPORT model (`role: support` in the manifest): `ModelManager::list()` leaves it out, so neither the UI engine pickers nor the default model list of `bench` ever contain it.
- Consequences: the VAD adds a small cost (detection time recorded) and one more model to ship in a product; cutting at silences can still split a sentence between two pieces, and each piece loses the context of the previous one (a name or a number spoken across a cut may be recognised differently); the VAD may miss quiet speech or treat noise as speech (one speaker, one microphone only). The 25 s limit and the VAD parameters were chosen a priori from the engines' documented window sizes, not tuned on the test clips; they are constants, easy to change if the owner wants an experiment.

### D-035 update (2026-10-08) — what the experiment showed
- Default stays OFF. Chunking at 25 s is useful for one engine only (sherpa-onnx Whisper tiny: recovers the ending its 30 s limit discards), neutral for Parakeet and whisper.cpp on the clips tested, and harmful for Canary (I-041). It does not remove Whisper repetition loops (I-018). The cut position was not the problem in any case seen; the engines' behaviour on pieces was.
- Before any product use: per-engine segment limit (a 25 s limit is wrong for Canary), an output guard (I-035), and a test on more speakers and longer dictations. Short audio (25 s or less) is provably unchanged (828 of 828 identical transcripts), so enabling the option never risks the short-query use case.
- D-032 (shortlist) is not changed by this experiment: it adds the note that Parakeet and whisper.cpp were the only engines without any long-audio failure here.

## D-036 — Drug-name and key-term handling: vocabulary biasing and dictionary post-correction (all OFF by default)
- Date: 2026-10-08 · Status: accepted as EXPERIMENTAL options; no technique is enabled anywhere; the product choice is the owner's (M8)
- Context: every engine misspells drug names (clean run: 4/6 found by the best engines, 1/6 to 3/6 by the others). A wrong or missing drug name is a critical error for a medical secretary assistant. Goal: which cheap technique reduces it, by how much, at what risk, on one speaker and CPU only.
- Options tested (results: project log 2026-10-08 T4): (A1) whisper.cpp initial prompt built from a vocabulary; (A2) sherpa-onnx hotwords on the NeMo transducer (Parakeet) with modified beam search; (B) dictionary post-correction of the text, three presets; (A+B) combinations. Not testable: Canary and sherpa-onnx Whisper (no biasing mechanism in this runtime).
- Decision (design):
  - The vocabulary travels with the request (`TranscribeRequest.vocabulary`, empty by default). Each provider decides how to use it (whisper.cpp: `--prompt "a, b, c."`; sherpa-onnx Parakeet: per-stream hotwords, only if the provider was built with `with_hotwords(score)`; others ignore it). The UI never names an engine. `RunRecord.biasing` records what happened ("initial prompt", "hotwords (score 1.5)", "ignored by engine").
  - Post-correction is a pure text function (`speech/postcorrect.rs`) behind any provider. It never touches numbers, units, negations, weekday/month names, plurals (a trailing s/x), words below the minimum length, or words that are themselves vocabulary terms; ties between two terms are left alone; every change is returned so a product can show or log it. Presets: STRICT (default if ever enabled), MEDIUM, LOOSE (study only). Applied offline to stored transcripts by `bench postcorrect`.
  - Vocabulary files `benchmark/vocab/<lang>.txt`: dataset key terms plus 16 common drug names as distractors (names only, no clinical content).
- Findings that drive the recommendation (one speaker, 95 sentences, a vocabulary taken from the test sentences, so an UPPER BOUND; only 6 drug occurrences):
  - STRICT post-correction: fixed 2 of 6 missing drug names for Parakeet (4/6 -> 6/6) and for whisper.cpp small and base, 0 words broken in all 9 configurations. Cheap, engine-independent, no latency. Cannot fix far misses ("libby profene").
  - whisper.cpp initial prompt: helps small/base on technical terms and overall WER (small 5 beams 3.9 -> 3.3 %), does not reliably fix drug names (small greedy lost one), slows inference by roughly 25 to 40 %, slightly hurts sentences without key terms, and clearly hurts tiny. Prompt + strict correction was the best combination on whisper.cpp small and base.
  - Parakeet hotwords: work only with a surrogate SentencePiece vocabulary derived from `tokens.txt` (the download has none). Score 1.5: one more drug found, WER 2.3 -> 2.7 %, loses the elision ("d amoxicilline"). Score 3.0: regression (WER 5.1 %, vocabulary words inserted in unrelated sentences, one runaway). Not recommended as it stands; post-correction gave a better result on the same engine.
  - LOOSE post-correction is a regression (WER Parakeet 2.3 -> 5.9 %; "matin"/"mais" turned into "main", "après" into "API REST").
- Recommendation (NOT a decision): if a drug-name safeguard is wanted, use STRICT post-correction with a curated vocabulary, log every replacement, and show it to the user; keep prompts/hotwords off until tested on more speakers; never use the LOOSE settings.
- Risks, stated plainly: a dictionary cannot tell a misspelling from a real, unusual word. A correct but rare word (a different drug one or two letters away: "prednisolone" against "prednisone" under the MEDIUM preset) is replaced silently, and in a medical text nobody re-reads a plausible word. Under STRICT the only guard is the vocabulary itself (list both drugs). A vocabulary file is also clinical content to maintain. Biasing the engine can insert vocabulary words into sentences that never contained them. All of this was measured on 95 short sentences from one speaker; the false-correction rate on free text is unknown.
- Consequences: new modules `postcorrect.rs`, `termstudy.rs`, a new request field, new `bench` options (`--vocab-dir`, `--hotwords-score`) and subcommands (`termstudy`, `postcorrect`), a probe example (`hotwords_probe.rs`). Results folders named `*-t4-*` are the evidence. No new dependency, no download.

## D-037 — Private clips are imported by a command that never prints the reference and refuses without consent
- Date: 2026-10-08 · Status: accepted
- Context: T5 adds recordings of other people (accents) to the benchmark. Their voices and texts are personal data; the dataset format already supports private samples, but creating one meant hand-writing a JSON and converting the audio by hand, which invites mistakes (wrong folder, committable audio, missing consent note, reference pasted into a chat or a log).
- Options: (a) hand-written JSON, as before; (b) extend the in-app recorder (UI work, not needed for existing files); (c) a `bench import` subcommand plus a library function that does the conversion, the placement and the validation.
- Decision: (c). `dataset::import_sample` converts to 16 kHz mono 16-bit, writes the audio and the metadata, validates with the normal dataset checks and rolls back on failure. Rules enforced in code: third-party or `--private` material always goes to `samples-private/` and is never committable; the consent statement is mandatory for private clips (`yes` or `unknown` is written into `source.license`, `no` refuses the import); public or licensed material needs a licence and a URL; an existing id is not overwritten without `--replace`; the reference is read from a file, cleaned (BOM, line breaks, repeated spaces) and never printed; error messages never contain it. The default category (`<language>-accent-private`) is unique to imported clips so that `--category` selects only them.
- Consequences: `bench import` is the supported way to add a clip; the owner types the reference, the tool never listens to or fixes it. Reports about private clips contain only aggregates and the words "this speaker". Known limits: only PCM WAV input; linear-interpolation resampling for clips that are not at 16 kHz (I-046); `summary.md` and `runs.jsonl` of a private run still contain the clip texts on disk (git-ignored folder) and must never be quoted (I-047).

## D-038 — Text-to-speech laboratory: one provider over sherpa-onnx, Piper first, Kokoro as a quality candidate, espeak-ng exposure accepted for the lab only
- Date: 2026-10-08 · Status: accepted for the lab (M6); product choice and the espeak-ng question stay open (M8, D-012)
- Context: M6 must tell whether sherpa-onnx can give usable French and English voices. Constraints: free and open-source, no non-commercial or copyleft voices without the owner's approval, offline synthesis, no auto-play, gender only when documented, naturalness is opinion.
- Options for voices: (a) Piper/VITS packages (small, fast, many languages, licence per dataset); (b) Kokoro multi-voice (better reputation, Apache-2.0 weights, one French voice); (c) Matcha/Kitten/Zipvoice/Pocket (English or gated, not examined); (d) the non-free candidates (excluded by rule).
- Decision:
  - Provider: `SherpaTtsProvider` in `speech/tts.rs` behind the existing `TextToSpeechProvider` trait; voices come from manifest entries with `role: "tts"` (no engine name in the UI). A voice id is `<package id>:<speaker id>`. TTS packages are kept out of `ModelManager::list()` so the STT pickers and `bench` are unaffected.
  - Candidates installed (owner-approved download, checksums pinned): Piper `fr_FR-siwis-medium` (CC-BY-4.0), Piper `fr_FR-gilles-low` (CC0), Piper `en_US-libritts_r-medium` (CC-BY-4.0), Kokoro multi-lang v1.0 int8 (Apache-2.0 weights; French voice trained on SIWIS, CC-BY-4.0). Excluded with reasons: Piper `fr_FR-tom` (AGPL), `vits-mms-fra` (CC-BY-NC), Piper `en_US-ryan` (CC-BY-NC-SA); not examined further: `en_US-lessac` (Blizzard licence page), `en_US-amy` and `en_GB-alan` ("See URL"), `fr_FR-miro` (no card), `fr_FR-upmc` (CC-BY-SA, share-alike, left for the owner).
  - Kokoro v1.1-zh (English and Chinese only) was installed by mistake and removed: French support must be read from the model's own card BEFORE a download is proposed.
  - Speed is passed to the library; it is a documented parameter, not an assumption: the measured effect differs by family (Kokoro linear, Piper compressed).
  - Cancellation uses the library callback; granularity is one sentence (`max_num_sentences = 1`).
  - Generated audio is written to `tts-output` next to the models folder, readable only from there, deletable from the UI; never played automatically.
- Licensing stance: the TTS path needs the full sherpa-onnx static libraries, which include espeak-ng (GPL-3.0) as phonemizer. This is accepted for the lab (nothing is distributed), and recorded as the blocking licensing question for any product use of TTS (D-012, I-003, I-010). The STT-only product path can still use the no-tts libraries. Model licences are per voice and need attribution (CC-BY) in any distribution.
- Findings that drive the recommendation (one machine, CPU, measured in the log): Piper is about 13 to 29 times faster than real time (RTF 0.03 to 0.08) and uses 210 to 260 MB; Kokoro int8 is slower than real time on this CPU (RTF 1.6 to 2.1) and uses 430 to 460 MB, so it cannot be interactive here; French has one Kokoro voice and three Piper voices examined. Machine intelligibility check: Kokoro `ff_siwis` retrieved 4 of 5 drug names with Parakeet, Piper 2 of 5 (anecdotal). Naturalness: the owner's listening notes are pending.
- Consequences: new module `tts.rs`, manifest fields `voices`/`lexicon`/`dataDir`, new commands and a UI panel, `examples/tts.rs`. Open: owner's listening notes, licence reading for the undecided voices, packaging of the phonemizer data (M7), Kokoro speed options (fp32, threads).

## D-039 — The TTS benchmark is licence-first: every voice is rated for commercial use, and only the least constrained voices can be recommended
- Date: 2026-10-08 · Status: accepted (owner's instruction) · Amends D-038
- Context: the product is meant to be used commercially. Voice licences differ widely and the card's "dataset licence" line alone is misleading (a CC0 dataset does not clear weights fine-tuned from a non-commercial voice). The first M6 selection was made on that line and was partly wrong.
- Decision:
  - Every voice package carries a rating in the manifest (`licenseTier` = `clear` / `attribution` / `review` / `excluded`, `licenseNotes` = the reason, `phonemizer` = what it needs at synthesis time). A unit test refuses a package without them. Definitions, evidence and the full table are in `docs/TTS_LICENSES.md` (living document, read dates recorded).
  - Rule for any new candidate: read the card including "Finetuned from / Trained from scratch", read the parent's licence, read the family's provenance statement, record the phonemizer, then rate. A permissive dataset line is never enough.
  - The rating is visible where the decision is made: voice package table, voice list, a warning under a `review` or `excluded` voice, and the listening page. Measurements are reported with it.
  - Benchmark order and scope: `clear` and `attribution` voices are the primary candidates and get the full measurement set and the owner's listening first; `review` voices are measured as references and always labelled; `excluded` voices stay installed only to keep earlier measurements comparable and are never recommended.
  - The final recommendation (M8) may name only `clear` or `attribution` voices. A `review` voice can be recommended only after the owner records, in this file, that a legal reading cleared it.
  - The phonemizer is rated apart (D-012, I-051, I-054): a licence-clean voice that needs espeak-ng still carries the GPL question; a character-based voice avoids calling it but needs a text normaliser (I-055).
- Corrections made by this decision: Piper `siwis` (fine-tuned from Lessac, research licence) moves from "CC-BY, usable" to `review`; Piper `gilles` (fine-tuned from Ryan, CC BY-NC-SA) moves from "CC0" to `excluded`; Kokoro stays `review` (training audio includes output of closed TTS services). The Kokoro speed statement of D-038 ("RTF 1.6 to 2.1") is replaced by "1.1 to 2.1 depending on the run".
- Candidates now: English Piper `ljspeech` (`clear`, installed), Piper `libritts_r` (`attribution`, installed), Piper `kristin` (`clear`, available in the release, not downloaded); French Coqui `css10` (`attribution`, installed, no phonemizer, needs a number normaliser). No `clear` French voice exists in the release; the cleanest Piper French voice (`mls`, CC BY 4.0, from scratch) needs a checkpoint conversion.
- Consequences: new fields and test, UI changes, `docs/TTS_LICENSES.md`, packaging (M7) must not ship a default voice rated `review` or `excluded`, and the credits for `attribution` voices must appear in the product. Open: I-052 to I-055.

## D-040 — Highlighting the text being read (PROPOSED, nothing built; the owner chooses the granularity)
- Date: 2026-10-08 · Status: proposed
- Owner's question: many reading applications put a grey background on the word, sentence or paragraph being spoken. Is it planned? (Granularity to be decided.)
- Facts: it is NOT in `Plan.md` (section C lists text input, voices, playback, pause/resume/stop, speed, WAV export, durations): a new requirement. The sherpa-onnx synthesis API returns samples and a sample rate only; its callback receives the samples so far and a progress value, never word or phoneme timings (checked in the crate source: the only timestamp options belong to speech recognition). The HTML audio element gives `currentTime` and free pause, resume, stop and seeking.
- Options:
  - A. Sentence level (and paragraph level as a lighter shade), EXACT. The application splits the text into sentences, synthesises each one (the library already works sentence by sentence), keeps each duration, joins the audio with a short silence into the same single WAV (so the player and "Save WAV" stay as they are), and highlights the sentence whose time span contains `currentTime`. Exact because the durations are measured, valid at any speed setting, click-to-seek per sentence is easy, and playback can start after the first sentence is ready, which helps voices slower than real time (Kokoro). Needs a sentence splitter that survives abbreviations ("Dr.", "M."), decimals and French typography, with tests. The text box becomes a read-only styled view while reading.
  - B. Word level, ESTIMATED. Spread each sentence's measured duration over its words by weight (letters or syllables, pauses at commas). Looks good, accuracy unknown (expect an error of the order of 100 to 300 ms, worse on long numbers and drug names): must be measured against a reference before it is trusted, and labelled approximate.
  - C. Word level, MEASURED. Align the generated audio with the text using a recogniser that outputs token timestamps (available in the library), or obtain durations from the voice model (the models predict them internally but the sherpa-onnx API does not expose them). Costs extra computation and is fragile on drug names and digits; a research task, not recommended now.
- Recommendation: build A first (sentence highlight with paragraph shading), then B only if the owner wants words and accepts "approximate"; do not promise C.
- Open questions for the owner: sentence, paragraph, or word granularity? Grey background only (light and dark themes) or also auto-scroll? Click a sentence to jump? Should reading start while later sentences are still being generated?
- Consequences if accepted: a splitter module with tests, per-sentence synthesis in the provider (timings per sentence in the result), a read-along view component, unit tests plus a UI test over the DevTools protocol.

### D-040 update (2026-10-08) — ACCEPTED by the owner, scope fixed
- Status: accepted for implementation (prompt `docs/prompts/06b-tts-readalong-and-normaliser.md`).
- Scope: SENTENCE-level highlight (option A) with a grey background that works in light and dark themes, and AUTOMATIC SCROLLING so that the sentence being read stays visible in long texts. Not in the first release: click on a sentence to jump to it (the owner sees it as a possible add-on or an editorial use case with a lot of text; keep sentence segments identified so it can be added), paragraph shading, word-level highlight (options B and C are not wanted now).
- Design constraints: highlight spans belong to the ORIGINAL text; any text normalisation is applied per sentence only for synthesis, so the mapping stays trivial. Playback still starts only when the user presses play.

## D-041 — Beta licence stance and French voice strategy
- Date: 2026-10-08 · Status: accepted (owner's decisions)
- Owner's stance: a beta version is acceptable only if it does not require a paid licence; otherwise abstain. A budget for legal advice or licences may be considered later, depending on the number of users and on revenue, once prices are known. Public-domain material is to be preferred at this stage.
- Consequences for voices: for the beta only `clear` and `attribution` voices may be offered by default; `review` voices (Piper siwis, Kokoro) and `excluded` voices (Piper gilles) stay installed as quality references only and are never recommended or shipped. No lawyer is consulted now (I-052 to I-054 stay open and cost-free to leave open).
- Listening result that makes this hard (owner's blind notes, `docs/TTS_LISTENING_NOTES.md`): the preferred French voices are `review`/`excluded`; the only freely usable French voice found (Coqui css10) scored lowest (clarity 3, accent 2) and does not speak digits. English is fine: the `clear` ljspeech and `attribution` libritts_r voices are among the owner's preferred.
- Decision for French: (1) build a French and English text normaliser (digits, dates, times, units, common abbreviations to words) and apply it to character-based voices (Coqui), so that voice is at least usable; its use is a per-voice manifest flag, not a hard-coded engine name; (2) search for a better freely usable French voice, in this order: read-only research of cards and releases, then a conversion of the Piper `fr_FR-mls-medium` checkpoint (CC BY 4.0, trained from scratch) for sherpa-onnx, which needs the owner's approval before any Python package is installed; (3) install `kristin` (done) and keep preferring public-domain data.
- Kokoro: deprioritised (rating `review`, slower than real time, hiss heard on all three voices, I-057). No further investment unless a clean quality problem needs it.
- Accent evaluation (T5): deferred; the owner has no clips. `bench import` stays ready; public material with a clear licence and accent metadata could replace private clips later, only on the owner's request.

## D-042 — Read-along, sentence splitter, text normaliser, per-sentence synthesis (M6d)
- Date: 2026-10-08 · Status: accepted by the owner's decisions D-040 (accepted) and D-041, implemented and tested
- Context: the owner wants the sentence being read shaded (grey, light and dark themes) with automatic scrolling for long texts, and the freely usable French voice (Coqui css10) must speak digits. The synthesis library returns samples only (no word or sentence timings), and the UI needs timings to drive a highlight.
- Decisions:
  1. **Timings are measured, not estimated**: the provider synthesises sentence by sentence (one library call each) and joins the samples into ONE WAV, so the player, "Save WAV" and cancellation stay as they were. `SynthesizeResult.segments` lists, in order, `{start, end, startMs, endMs}`: `start`/`end` are offsets in the ORIGINAL request text in UTF-16 code units (the unit of JavaScript string indices, so `text.slice(start, end)` is the sentence; Rust byte offsets would break on accents), `startMs`/`endMs` the sentence's speech in the WAV (before the pause that follows). Segments carry their index so a later click-to-jump needs no new data (exposed as `data-segment`, `data-start-ms`, `data-end-ms` on the spans).
  2. **Silences**: 250 ms after a sentence, 600 ms after a paragraph (blank line), no pause after the last sentence, both divided by the speed factor (a 2x reading has 125 ms and 300 ms). Chosen by ear-neutral rule of thumb, not tuned; constants `SENTENCE_PAUSE_MS` and `PARAGRAPH_PAUSE_MS` in `tts.rs`. The library's own inter-sentence silence is no longer used (one call = one sentence). The added pauses make the audio 7 to 9 % longer on 5 and 10-sentence texts.
  3. **Splitter** (`sentences.rs`): `. ! ? …` end a sentence when followed by a space or the end and the next visible word does not start with a lower-case letter (so "Je pense... peut-être" and "« Tu viens ? » demanda-t-il." stay whole); closing quotes and brackets belong to the sentence, a guillemet may follow a space (French typography); known abbreviations, a lone capital (initial) and a list number at the start of a sentence never end it; "etc." ends it only before a capital; a line break always ends a sentence, a blank line ends the paragraph. Known cost: a single line break inside a hard-wrapped paragraph also splits (the text box is for dictated or typed text, where a line is an item). Pieces without a letter or digit are glued to the previous sentence on the same line or dropped.
  4. **One source of truth**: `plan_sentences(text, language, normalise)` produces for each sentence its original span and the text the voice receives; the synthesis, the "text sent to the voice" preview (command `preview_spoken_text`) and the read-along all use it. A request with one sentence behaves as before (one segment). `synthesize_whole_text` (one library call for the whole text) exists only so the overhead can be measured in the same run.
  5. **Normaliser** (`normalise.rs`, a hand-written scanner, no regex dependency): applied per sentence and ONLY to the spoken text, never to the displayed text; switched on per voice package by the manifest flag `normalizeText` (true for Coqui css10, false for phonemizer voices; a unit test requires the flag to equal "phonemizer starts with none"); the request can override it and the UI shows a checkbox (default from the voice) plus the preview, for transparency. French: 70 / 80 / 90 as "soixante-dix", "quatre-vingts" (plural s only when nothing follows), "quatre-vingt-un", "et un / et onze", cents and mille, "un" becomes "une" only before a short list of common feminine nouns, ordinals from the cardinal ("vingt et unième", "quatre-vingtième"), "de/d'" after millions. Decimals use "virgule" (French, decimals up to three digits as a number, leading zeros one by one) or "point" (English, digit by digit). Formats chosen: thousands separators are space or a dot followed by exactly three digits (French) or a comma (English); "10.30" is a time only after a time word (à, vers, entre, ...) and not before a unit or an amount (I-034 relation: the French decimal separator is the comma); "14 h 30", "9h00", "14:30" are always times (minutes 00 are dropped); "12/03/2026", "12.03.2026", "2026-03-12" are dates (French: day first; English: month first unless the first number exceeds 12); after a date word, "le 12/03" is a day and a month; other "x/y" are fractions ("un demi", "trois quarts") or "x sur y"; units follow the plural rule of the language (French plural from 2, English plural unless exactly 1); rates "mg/kg", "/jour" become "par"; **phone numbers: groups of exactly two digits are read as numbers ("06" = "zéro six", "78" = "soixante-dix-huit"), any other group digit by digit, commas between groups; identifiers (9 digits or more in several groups, or a long plain digit string) digit by digit**; "numéro 2045" and other short numbers after "numéro" are read as ordinary numbers.
  6. **Highlight in the interface**: the view is a read-only block of sentence spans (a textarea cannot style parts of its text); the active index comes from `audio.currentTime` on every animation frame (`requestAnimationFrame`, not the 4 Hz `timeupdate`) through the pure function `activeSegmentIndex` (a sentence stays active during the pause after it); nothing is shaded before the first play, when stopped (paused at 0) or after the end; scrolling is smooth, goes only when the sentence is not comfortably visible (the sentence is put in the upper third), is suspended for 4 s after a manual scroll and can be turned off with "Follow reading"; the block sets its own text and background colours (light: grey `#d4d4d4` with `#111`; dark: `#4d4d4d` with `#fff`) so the contrast holds whatever the page does.
  7. **Converted voices**: a new packaging kind `local` for a voice built on this machine by `scripts/convert_piper_voice.py` (Piper `fr_FR-mls-medium`): `install` refuses it before touching anything (the folder holds the converted voice); `archiveUrl` and `sha256` describe the SOURCE file for provenance.
- Options not chosen: word-level highlight and paragraph shading (owner: not wanted now); a click on a sentence to jump (owner: maybe later; the data are there); estimating sentence times from the text length (rejected: measured durations are exact at any speed); normalising inside the voice package or with a regular-expression crate (rejected: more dependencies, less control over French plurals); reading an invoice number digit by digit (an ordinary number is clearer to a listener; the rule is one line to change).
- Consequences: the TTS contract has two new fields (`segments`, `normalised`) and one optional request field (`normalise`); a per-sentence synthesis costs a few percent of generation time (measured, run `20261008-141002-tts-m6d-overhead`); Coqui speaks numbers (rough recogniser check: 0/36 key numbers heard without the normaliser, 28/36 and 30/36 with it) but its articulation of other words remains weak (I-058); the English rules have unit tests but no character-based English voice to hear them; known limits listed in I-061.

## D-043 — French voice with a clean lineage; rewriting on by default for every voice (ACCEPTED by the owner, 2026-10-08, in this form: see "Owner's decision")
- Date: 2026-10-08 · Status: accepted by the owner (the proposal below is kept as written; the decision follows it)
- Context: after the interface tests and listening session 2 the owner's judgement is that the good French voices are Piper siwis (female, `review`) and gilles (male, `excluded`), and that every French voice allowed by D-041 is unsatisfactory (Coqui css10 with the normaliser 3 out of 5, converted `mls` rejected). In English three voices are fine (libritts_r, kristin, ljspeech). Evidence: `docs/TTS_LICENSES.md`, section "French voice: lineage problem and ways out".
- The problem is the PARENT checkpoint, not the data: siwis was fine-tuned from Lessac (Blizzard 2013 research licence), gilles from Ryan (CC BY-NC-SA 4.0). Whether weights inherit such a restriction is untested law (the Piper maintainer says so himself) and he announced re-training voices from a CC BY base. That base is public (`_base_model`, trained from scratch on LibriTTS-R, 933 MB).
- Options: (1) re-train siwis and gilles from the clean base (needs a GPU outside this machine; rating would become `attribution`; quality and cost NOT VERIFIED); (2) ask the rights holders for a written statement; (3) accept the risk for siwis (excluded by D-041); (4) beta with English voices and Coqui for French meanwhile. Recommendation: (1) with (4) as the interim, and one comparison run with the French `mls` checkpoint as the parent.
- Second proposal, from the owner's own use: the owner switches the rewriting of numbers, dates, times and units ON for the phonemizer voices too. Make it the default for every voice (manifest `normalizeText` true everywhere, the unit test that ties it to the phonemizer field removed), keeping the checkbox. Small change, not done yet.
- Consequences if accepted: a training guide and data preparation scripts in the repository, a hosting question for converted voices (I-060), a new blind listening session (reference: the current siwis and gilles).

### D-043 — Owner's decision (2026-10-08)
- **Keep Piper siwis (female) and gilles (male) as the French voices now. Re-train them from the clean base checkpoint LATER, when the software is to be commercialised.** The owner's reasons, as stated: they are the only French voices judged good (listening session 2 and interface tests); a commercial release is not for now.
- **Number rewriting (numbers, dates, times, units in words) is ON by default for every voice**: approved and CODED (manifest `normalizeText` true for all 8 voices, the unit test in `models.rs` now requires it, the checkbox kept, the request can still override it).
- **What this changes**: D-041 said that only `clear` and `attribution` voices are offered by default and that `review` and `excluded` voices are never shipped or recommended. For siwis (`review`) and gilles (`excluded`) that rule is SUSPENDED until commercialisation (D-041 is amended, not cancelled: it applies again to any commercial release). The ratings in the manifest and in the app stay `review` and `excluded`, so the app keeps showing the warning; nothing about their lineage was resolved (I-052, I-063).
- **Gate before any commercial release**: replace siwis and gilles by voices re-trained from `rhasspy/piper-checkpoints/_base_model` (or get a written statement from the rights holders, or a legal reading), re-listen blind, update the ratings. Until then the two voices are for development, evaluation and non-commercial use only.
- **Open point to confirm with the owner**: a beta handed to other people, even free of charge, is a distribution of the two voices (the models are downloaded from the app, not bundled, but the app offers them); whether that counts as "non-commercial use" is a legal question this decision does not settle. Models are not bundled in any case (M7).
- Not decided: when, where and with what GPU the re-training happens; which parent (`_base_model` or the French `mls` checkpoint); paid French voices (prices in `docs/TTS_LICENSES.md`, none chosen).

## D-044 — Windows packaging: per-user NSIS installer, whisper.cpp as a sidecar with its four libraries, models never bundled (M7)
- Date: 2026-10-08 · Status: accepted by the owner (format and sidecar choice), built and verified on this machine
- Context: until M7 everything ran from `pnpm tauri dev` and `cargo` examples; `bundle.active` was `false`. The Tauri tools needed to package were already in `%LOCALAPPDATA%\tauri` (NSIS and WiX), so nothing had to be installed. The whisper.cpp command line tool is built as shared libraries: VERIFIED that `whisper-cli.exe` does not start without any one of `ggml.dll`, `ggml-base.dll`, `ggml-cpu.dll` and `whisper.dll` (exit code 127 with a missing library, exit 0 with the four; exe + libraries = 3.3 MB). sherpa-onnx is linked statically into `speechlab.exe` (31.4 MB), so no sherpa library is needed next to it.
- Options for the installer: NSIS `.exe` only, MSI only, both. For the sidecar: ship the four libraries as bundle resources next to `whisper-cli.exe` (no recompilation), or rebuild whisper.cpp as one static executable (a new CMake build and a change of `scripts/build-whisper-cpp.ps1`).
- Decision: **NSIS `.exe` only, installation mode `currentUser`** (no administrator rights, `%LOCALAPPDATA%\<product>` by default). MSI stays possible later (the WiX tools are cached). **`whisper-cli.exe` is a Tauri `externalBin` sidecar** (`binaries/whisper-cli`, staged as `whisper-cli-x86_64-pc-windows-msvc.exe`) and the four libraries are `bundle.resources` mapped to the install root (`"binaries/ggml.dll": "./"` and so on), so that all six files sit next to `speechlab.exe`. This is exactly where `locate_cli` in `whisper_cpp.rs` already looks first (next to the executable), so no Rust change was needed. `scripts/stage-whisper-sidecar.ps1` copies the files from the git-ignored `vendor/` build into the git-ignored `src-tauri/binaries/`.
- **What the installer carries**: `speechlab.exe`, `whisper-cli.exe`, the four libraries, `uninstall.exe` (7 files, 34,918,850 bytes installed; the installer is 8,858,030 bytes = 8.45 MiB). **What it does not carry**: any speech model, any voice package, the `espeak-ng-data` folders (they are inside each downloaded voice package), any test audio. Models keep living in `%APPDATA%\ai.assistantcabinet.speechlab\models`, so an upgrade or an uninstall does not touch them (VERIFIED for the silent uninstaller, see the log). The owner's rule "models are never bundled" holds.
- Consequences: (1) every `cargo build`, `cargo test` and `cargo run --example` of the `speechlab` package now needs the staged files, because `tauri-build` checks `externalBin` (VERIFIED: without them the build stops with "resource path ... doesn't exist"); run the staging script after `scripts/build-whisper-cpp.ps1` (I-064). (2) A development run copies the same six files next to `target\debug\speechlab.exe` (VERIFIED), so `pnpm tauri dev` keeps working. (3) The installer, `speechlab.exe` and the sidecar are NOT signed (I-068). (4) The espeak-ng phonemizer (GPL-3.0) is inside `speechlab.exe`; the installer of this build therefore carries the licence obligation of D-012 / I-051 / I-054, which this decision does not settle. (5) The application shows each voice's licence and commercial-use rating in the TTS table but has no credits view and the installer shows no licence page (I-067); the attribution voices (CC BY 4.0, BSD-3-Clause) and the GPL notice still need a place before any distribution. (6) Siwis and gilles remain development-only voices until re-trained (D-043, I-063): they are not bundled and a commercial build must not offer them by default. (7) The sidecar and its libraries import the Visual C++ runtime that the installer does not carry (I-070): the static rebuild of whisper.cpp that was NOT chosen here would remove both the four libraries and that dependency, and is the recommended follow-up before any distribution.
- macOS: not built (no Mac). The steps are in `docs/MACOS_VALIDATION.md`, all NOT VERIFIED.

## D-045 — Content Security Policy for the packaged app (M7, approved by the owner)
- Date: 2026-10-08 · Status: accepted by the owner (the CSP; the other proposed code changes were not chosen), built and verified
- Context: `app.security.csp` was `null` (a development setting): the page could load anything. The page uses no external resource: its only network use is inside the Rust code.
- Decision: a strict policy for the release build: `default-src 'self'; script-src 'self' blob:; worker-src 'self' blob:; style-src 'self'; img-src 'self' data: blob:; media-src 'self' blob: data:; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`. A separate `devCsp` (adds `'unsafe-inline'` for the Vite and React development scripts and styles, and `ws://localhost:1430` / `http://localhost:1430` for the development server) keeps `pnpm tauri dev` working. `blob:` is in `script-src` because the microphone recorder loads its AudioWorklet from a blob URL (`src/audio/recorder.ts`): the first attempt without it failed exactly there ("Failed to load worklet module script: blob:..."), VERIFIED.
- Verified: in the packaged app the policy is enforced (an inline script added to the page is blocked and raises a `script-src-elem` violation); all UI checks pass with it (33 of 33 read-along checks, and the packaged-app test except the already known cancel banner, I-065), including TTS playback, the microphone recording and both speech-to-text engines; in development the 33 read-along checks also pass with `devCsp`.
- Not chosen / later: serving the worklet as a static file of the application (`script-src 'self'` without `blob:`) is a small code change that would tighten the policy further; `style-src 'self'` works because React sets styles through the DOM API, no inline `style` attribute is in the HTML. The capability file (`core:default` only) was read and left unchanged: narrowing it was not requested. Both are candidates for a later hardening step.
