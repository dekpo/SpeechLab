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
