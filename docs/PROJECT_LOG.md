# Project log

Append-only journal, newest entry at the bottom. Record what was done, commands, results, **failures, bugs, and dead ends**. Written in English. See `AGENTS.md` for the rules.

Entry template:

```
## YYYY-MM-DD — <milestone> — <title>
Done: ...
Commands / evidence: ...
Worked: ...
Failed / surprises: ...
Open questions: ...
Next: ...
```

---

## 2026-10-06 — M0 — Feasibility and dependency validation

**Done**
- Inspected the machine and toolchain (`scripts/check-env.ps1`).
- Queried GitHub, crates.io, and Hugging Face APIs for versions, licenses, release assets, and model lists.
- Wrote `docs/M0_FEASIBILITY.md` (engines, models, licenses, architecture, plan).

**Worked**
- Machine: Intel Core 7 150U, 23.6 GB RAM, Windows 11 x64, no discrete GPU → CPU-only benchmarks.
- Present: Node 24.19, npm 11.17, pnpm 12.4, Rust 1.98.1 (msvc), MSVC Build Tools 14.44, Windows SDK 10.0.26100, WebView2 154, Git, Python 3.13.
- Missing: CMake, LLVM/libclang (only needed for whisper.cpp builds).
- sherpa-onnx v1.13.8: prebuilt Windows/macOS libraries and an official `sherpa-onnx` Rust crate exist. French STT/TTS model archives exist.
- whisper.cpp v1.9.4: no prebuilt Windows binaries in the latest GitHub release → source build required.

**Failed / surprises**
- Two Hugging Face model pages returned HTTP 401 (`pierre-cheneau/finetuned-kroko-fr`, `csukuangfj/sherpa-onnx-streaming-zipformer-fr-2023-04-14`) → licenses of those French streaming models are still unknown.
- Piper model cards for `fr_FR/miro/high` and `fr_FR/tjiho/model3` returned 404 → licenses unknown.
- My first environment script printed the MSVC folder name instead of its version (wrong glob depth). Fixed. See ISSUES I-001.
- The `python` on this machine is the Microsoft Store build (`WindowsApps`). Works, but note for scripts.

**Licensing findings that shape the project**
- espeak-ng (GPL-3.0) is used by sherpa-onnx's Piper/VITS/Kokoro TTS path → high commercial-distribution risk (see DECISIONS D-004).
- Piper `fr_FR-tom` is AGPLv3; `vits-mms-fra` is CC-BY-NC → both excluded.

**Open questions**
- Does a commercially usable, good-quality French male voice exist?
- Swiss-French test data source (none identified yet).
- Microphone access inside Tauri WebViews (WebView2, macOS WKWebView).
- Can sherpa-onnx offline recognition be cancelled mid-decode?

**Next**: M1 (minimal Tauri 2 app + provider contract), sherpa-onnx first. See D-002.

---

## 2026-10-06 — M0 — Project governance and documentation set-up

**Done**
- Added `AGENTS.md` (global rules: git approval, identity rule, documentation rules, constraints).
- Added `docs/GIT_WORKFLOW.md`, `docs/DECISIONS.md`, `docs/ISSUES.md`, this log, `README.md`, `.gitignore`.
- Owner decisions recorded: start with sherpa-onnx, free/open-source tools first, owner runs all commit/push/PR commands.

**Worked**: files created; no git repository exists yet (owner runs `git init`, see `docs/GIT_WORKFLOW.md`).

**Failed / surprises**: none.

**Next**: owner reviews and commits M0; then M1.

---

## 2026-10-06 — M1 — Minimal Tauri 2 application and provider contract

**Done**
- Hand-written scaffold (no generator): Vite 8 + React 19 + TypeScript 7 frontend, Tauri 2.12.1 Rust backend in `src-tauri/`.
- Engine-agnostic contract: Rust traits `SpeechToTextProvider` / `TextToSpeechProvider` (TTS declared only), types in `src-tauri/src/speech/types.rs`, mirrored in `src/speech/types.ts`.
- `ProviderRegistry` + a clearly labelled **mock** STT provider (`is_mock = true`, performs no recognition) to prove the UI → Rust → provider round trip.
- Tauri commands: `list_stt_providers`, `transcribe` (runs on a blocking worker), `cancel_transcription` (cooperative `CancelToken`).
- Minimal UI: engine select, explicit language select (fr/en), WAV path input, Transcribe/Cancel, mock warning banner.
- Placeholder app icon generated with a small Python script, then `pnpm tauri icon` (Android/iOS icon folders deleted: not needed).

**Verified (this machine, Windows 11 x64)**
- `pnpm typecheck` passes; `pnpm build` (vite) succeeds.
- `cargo test` in `src-tauri`: 4 tests pass (registry lists mock flagged as mock, unknown provider error, empty path + cancellation behaviour, camelCase JSON contract). First Tauri compile took about 4 min.
- `pnpm tauri dev`: native window "SpeechLab" opens (process running, WebView2 content served), and a screenshot shows the provider list "Mock STT (no recognition) (mock)" with languages and capabilities → the `list_stt_providers` command round trip works UI → Rust → UI.

**Failed / surprises**
- `pnpm tauri dev` first failed: **port 1420 was already used** by the dev server of the separate AssistantCabinetAI desktop app (`apps/desktop`). Not touched. SpeechLab now uses port **1430** (see ISSUES I-006, DECISIONS D-008).
- One multi-file shell heredoc command failed with a bash parse error and wrote nothing; the files were re-created with the file-write tool. No impact on the repo.
- pnpm 12 automatically created `pnpm-workspace.yaml` with `minimumReleaseAgeExclude: [vite@8.3.3]` (pnpm's recent-release protection). Kept as generated; see ISSUES I-007.
- Vite 8 and TypeScript 7 are very recent major versions; they worked here without problems, but tooling/plugins may lag. Pinned via caret ranges and `pnpm-lock.yaml`.
- Dev-mode file watcher printed `EBUSY` warnings for files inside `src-tauri/target` while rebuilding; harmless, the app still started.

**Not verified**
- Clicking "Transcribe" in the window (the `transcribe` command is only covered by Rust unit tests, not by a UI click).
- `pnpm tauri build` / packaging (bundling is disabled, scheduled for M7).
- macOS: not testable here.

**Next**: M2 — sherpa-onnx STT adapter (official `sherpa-onnx` crate), model download manager, French and English WAV transcription offline.

---

## 2026-10-06 — M2 — sherpa-onnx speech-to-text adapter

**Done**
- Added crates: `sherpa-onnx` =1.13.8 (official), `hound` (WAV), `ureq` (+`native-certs`), `bzip2`, `tar`, `sha2`.
- `src-tauri/models-manifest.json`: model inventory as data (id, languages, architecture, quantization, size, license, source URL, runtime, platforms, SHA-256, files). New models can be added without code changes (new model *families* need an adapter branch).
- `speech/models.rs` (model manager: inventory, install status, install), `speech/download.rs` (the only network code: download, SHA-256 check, extract), `speech/wav.rs` (any WAV to mono f32; resampling left to sherpa-onnx), `speech/sherpa.rs` (adapter for Whisper, Canary, NeMo transducer families; recognizer cache keyed by model+language to measure cold vs warm).
- Commands: `list_models`, `install_model` (progress events), `cancel_model_download`; `transcribe` now returns load time, cold/warm flag, RTF, threads.
- UI: model inventory table with Install button and progress, model select, editable transcription, timing line.
- CLI example `src-tauri/examples/transcribe.rs` (`list`, `install`, `run <model> <fr|en> <wav> [repeat]`) for headless checks.
- `scripts/fetch-sherpa-libs.ps1` (see I-009). Mock provider removed from the default registry (D-009), kept for tests.

**Verified (this machine: Intel Core 7 150U, 23.6 GB, Windows 11 x64, CPU only, 4 threads, debug build)**
- 13 Rust unit tests pass (manifest, registry, WAV decode/mixdown, download helpers); `pnpm typecheck` passes.
- Model install through the app's own downloader: Canary int8 archive SHA-256 matched the value published by GitHub; Whisper tiny hash was recorded on first download (no published digest) and pinned; re-install from the UI re-verified it.
- Offline-capable inference path by construction: `sherpa.rs` has no network code (network only in `download.rs`). A formal disconnected-network test is NOT yet done (M8).
- Real transcription, official sherpa-onnx sample `fr.wav` (4.97 s, French): Canary int8 gave "Ne vous demandez pas ce que votre pays peut faire pour vous. Demandez vous plutôt ce que vous pouvez faire pour lui." Whisper tiny gave the same sentence except "Demandez vos puto". English samples from the model archives were transcribed correctly by both.
- Full end-to-end in the real Tauri window (driven through the WebView2 DevTools protocol on localhost): model table from the Rust backend, UI install with progress events (phases download/extract observed), transcription of `fr.wav` with Canary (inference 757 ms, RTF 0.152, cold load 1158 ms), Whisper tiny EN via UI (RTF 0.070), and a missing-file error shown in the UI.
- Observed timings (single machine, single sample, NOT benchmark data): Canary fr RTF 0.15 to 0.53 across runs; Whisper tiny RTF 0.07 to 0.14. Run-to-run variance is large (Canary 1306 ms vs 2651 ms on the same input, warm), so M5 needs many repetitions and recorded system state.
- espeak-ng finding (important, see D-012): the default static link of sherpa-onnx puts espeak-ng (GPL-3.0) into the STT-only executable (65 matches for "espeak" in the binary). A build against the official `no-tts` static libs contains 0 matches and transcribes identically (Canary fr output identical).

**Failed / surprises**
- First `cargo test` failed: the sherpa-onnx-sys build script download failed with `invalid peer certificate: UnknownIssuer`. Cause: Avast Web/Mail Shield intercepts TLS on this machine; its root CA is in the Windows store but not in rustls' bundled roots (I-009). Workaround: download with `curl.exe` (Windows trust store), verify SHA-256 against GitHub's digest, point `SHERPA_ONNX_ARCHIVE_DIR` at it via a git-ignored `src-tauri/.cargo/config.toml`. The runtime downloader uses `ureq` with `native-certs`, which works.
- Another multi-file shell heredoc failed with a bash parse error (nothing written); used the file-write/edit tools instead.
- First PowerShell scan of the no-tts binary printed zeros because `[Text.Encoding]::Latin1` does not exist in Windows PowerShell 5.1 (invalid result, redone with codepage 28591). Recorded so those zeros are not trusted.
- My first UI-driver script had wrong argument indexes (the screenshot path was used as the WAV path); fixed. Test tooling only, not product code.
- sherpa-onnx prints resampler messages to the console from its C++ code. Harmless, but noisy for CLI use.
- sherpa-onnx offline decode is a blocking call; there is no cancel hook. Cancellation only works before a run starts (reported honestly via `supportsCancellation: false`).
- The Canary archive ships no French test wav; the French sample is the official `fr.wav` from the sherpa-onnx `asr-models` release (not committed; stored in git-ignored `wav/`). Its license is not stated, so test use only, never part of the benchmark dataset until verified.

**No-TTS experiment (reproduce)**: download `sherpa-onnx-v1.13.8-win-x64-static-MT-Release-no-tts-lib.tar.bz2` from the official release, extract its `lib/`, add empty placeholder `espeak-ng.lib`, `piper_phonemize.lib`, `ucd.lib` (the crate's static link list always names them), then build with `SHERPA_ONNX_LIB_DIR=<that lib dir>` and a separate `CARGO_TARGET_DIR`: `cargo build --example transcribe`. It linked and ran. This is a workaround, not a supported configuration of the crate; TTS (M6) needs the full libs, so a TTS-capable binary would still contain espeak-ng.

**Not verified**
- Accuracy and speed conclusions (single samples only; the benchmark is M5).
- Parakeet TDT v3 model (listed in the manifest, file names taken from upstream docs, not downloaded or tested).
- Peak memory, CPU utilisation, release-build timings.
- Swiss French, accents, medical/legal vocabulary (M5).
- macOS; offline test with the network physically disabled (M8); `pnpm tauri build` (M7).

**Next**: M3 — whisper.cpp adapter (needs CMake; LLVM only if using `whisper-rs`). Ask the owner before installing.

---

## 2026-10-06 — M2 — Owner manual tests (informal, owner-reported)

Tester: the owner, in the real app, on their own recordings. These are anecdotal observations, not benchmark data: no reference transcript, no repetition, recording conditions and exact audio not recorded.

**Results**
- Official sample `wav/fr.wav`: works well. Canary int8 does NOT make the "puto" error that Whisper tiny made (consistent with the earlier CLI/UI runs).
- Owner's own French recording: Canary int8 results "assez bons" (fairly good). Whisper tiny "re-transcribes very badly" on the same test.
- `wav/Coucou.wav` failed with `audio error: cannot open ...: Ill-formed WAVE file: no RIFF tag found`.

**Diagnosis (VERIFIED by reading the file header, read-only)**
- `Coucou.wav` starts with `ftypisom`: it is an MP4/M4A container renamed to `.wav`, not a WAV file. The WhatsApp voice note `.ogg` in `wav/` is Ogg (`OggS`), also not WAV. The error message is technically correct but unhelpful (see I-015).
- Phone and messaging voice notes (M4A/AAC, Ogg/Opus) are a likely real-world input, so format handling matters for M4 (recording and import).

**Interpretation (tentative, to be confirmed in M5)**
- The tiny Whisper model is much weaker than Canary 180M on French in these informal tests. This matches expectations for the smallest Whisper size but is NOT yet a measured result. Larger Whisper models (base/small/turbo) have not been tried and must be included before concluding anything about "Whisper" as an engine family.

**Owner decisions (2026-10-06)**: install CMake via winget for M3; add a clear error for non-WAV files now (full decoding later).

**Done after the decisions**
- `wav.rs`: magic-byte detection of MP4/M4A, Ogg, FLAC, MP3, WebM, RF64 with an explicit message. 15 unit tests pass. Verified on the owner's real `.ogg` file.
- After the owner converted `Coucou.wav` to a real WAV, a CLI run with Canary int8 worked on the 45.8 s recording (RTF 0.455, debug build). Canary logged an end-of-text fallback warning (I-017). The transcript contains private content and is deliberately NOT stored in the docs.

**Next**: M3 (whisper.cpp), starting with installing CMake.

---

## 2026-10-06 — M3 — whisper.cpp adapter (external process)

**Done**
- Installed CMake 4.4.4 with `winget install Kitware.CMake` (owner approved). LLVM was NOT needed (D-017).
- `scripts/build-whisper-cpp.ps1`: clones the official repo at tag v1.9.4 (commit 927cfce) into git-ignored `vendor/whisper.cpp` and builds `whisper-cli.exe` (Release, CPU) with CMake + `NMake Makefiles` inside the MSVC environment. Build time 91 s. The Visual Studio generator was not used because `vswhere.exe` is absent on this machine (see I-022).
- `speech/whisper_cpp.rs`: `WhisperCppProvider` runs `whisper-cli` as a child process (`-m model -f wav -l lang -t 4 -bs N -bo N -nt`), parses whisper.cpp's own timing report (load / total), joins the transcript lines, supports real cancellation (the process is killed), no console window flash on Windows.
- `models.rs` generalised: single-file models (`packaging: "file"`) besides archives; `files` fields are optional so a bad manifest returns an error instead of panicking. Four ggml models added to the manifest with the SHA-256 published by Hugging Face: tiny (fp16, 78 MB), base q5_1 (60 MB), small q5_1 (190 MB), large-v3-turbo q5_0 (574 MB). Only tiny was downloaded.
- New result field `decoding` (e.g. "beam search (5 beams)", "greedy search") shown in the UI and CLI, so engine comparisons are never silently confounded by decoding strategy.
- CLI example now picks the engine from the model's provider; `SPEECHLAB_WHISPER_BEAM_SIZE=1` gives greedy decoding.

**Verified (this machine, CPU only, 4 threads, debug Rust build, whisper.cpp Release build)**
- 21 Rust unit tests pass (incl. timing parser, transcript cleanup, and a test that cancellation kills a real child process promptly). `pnpm typecheck` passes.
- whisper.cpp tiny ggml installed through the app's manager; SHA-256 matches Hugging Face's published value.
- Real transcription of the official `fr.wav` (4.97 s): whisper.cpp tiny returns "Ne vous demandez pas ce que votre pays peut faire pour vous. Demandez-vous plutôt ce que vous pouvez faire pour lui." with 5 beams AND with greedy decoding. RTF about 0.23 to 0.27 (5 beams), 0.15 to 0.20 (greedy). English sample correct.
- Same file, same tiny weights through sherpa-onnx (greedy): "Demandez vos puto" in 3 of 3 runs. So the decoding strategy does NOT explain the difference (whisper.cpp greedy is correct). Open hypotheses (not tested): different resampling of the 22.05 kHz file, sherpa's ONNX export/feature extraction, whisper.cpp's temperature fallback and token suppression. Single sample.
- End to end in the real Tauri window (WebView2 DevTools driving, localhost only): both engines listed; whisper.cpp tiny transcription of `fr.wav` from the UI (inference 747 ms, RTF 0.150, model load 87 ms); Cancel click during a long run returned "operation cancelled" within the driver's ~1 s polling granularity, and no `whisper-cli` process was left running.
- Owner's own 45.8 s recording (private content NOT reproduced here), CLI, one run each: sherpa Whisper tiny RTF 0.116 but its text ends in a repetition loop ("c'est bon, c'est bon, ...", a classic greedy-decoding hallucination), 71 words; whisper.cpp tiny 5 beams RTF 0.161, 100 words, no loop; whisper.cpp greedy RTF 0.048, 96 words; Canary int8 RTF 0.399, 99 words. Word-level disagreement with Canary's output (NOT a ground-truth WER, Canary can be wrong too): sherpa Whisper tiny 0.95, whisper.cpp tiny 0.65, whisper.cpp greedy 0.70.
- This probably explains the owner's "Whisper tiny transcribes very badly" report: it was sherpa-onnx's Whisper tiny on a recording longer than 30 s, which looped. It does not show that whisper.cpp tiny is good: its disagreement with Canary is still large.
- Canary's transcript ends earlier than whisper.cpp's ("... je pense." vs whisper.cpp continuing with a closing sentence). Canary may have DROPPED the end of the recording (it also logged the end-of-text warning, I-017). Needs the owner to listen to the end of the file (I-019).

**Failed / surprises**
- `cargo test` initially failed to compile because `DecodedAudio` lacked `Debug` (needed by a new test); fixed.
- A first start of the dev app failed with "Port 1430 is already in use" (a leftover dev server from earlier steps); stopped only SpeechLab's own processes and restarted. 1420 (the other app) untouched.
- Mid-step the shell tool again rejected a large multi-heredoc command; used the file-write/edit tools.
- whisper.cpp v1.9.4 also builds a `parakeet-cli` (NVIDIA Parakeet in whisper.cpp). Not evaluated; noted as a possible later experiment (model license to check).
- Every whisper.cpp run is a cold start (new process, model reloaded): this is a property of the external-process design (I-020). The benchmark must report load and inference separately (already the case).

**Not verified**
- Accuracy ranking of anything (single samples; M5 with references).
- whisper.cpp base/small/turbo models (in the manifest, not downloaded); GPU/Vulkan builds (CPU only by design here).
- Packaging whisper-cli inside a Tauri bundle (sidecar) and macOS builds (M7).
- Peak memory and release-build Rust timings.

**Owner answer (2026-10-07)**: the recording really ends with a closing sentence (goodbye + the addressee's first name + "ciao"), so Canary int8 DID drop the end of the 45.8 s file (I-019 confirmed). whisper.cpp tiny kept the sentence but misheard the name. First name not recorded here (personal data).

**Git incident (owner, 2026-10-07)**: `git switch main` was refused (uncommitted M3 changes would be overwritten by the checkout), and the following `git merge` then ran on the M2 branch itself ("Already up to date"). Nothing was lost or pushed wrongly; `main` still only had the first commit because M0, M1 and M2 had never been merged. Fix: commit M3 on its own branch first, then merge the milestone branches into `main` in order. `docs/GIT_WORKFLOW.md` now says so.

**Next**: M4 — audio capture from the microphone, WAV import polish, engine comparison view (same recording through both engines side by side).

---

## 2026-10-07 — M4 — Microphone, audio import and engine comparison

**Done**
- Microphone capture inside the WebView (`src/audio/recorder.ts`: getUserMedia + AudioWorklet, echo cancellation / noise suppression / auto gain OFF), resampled to 16 kHz mono 16-bit WAV (`src/audio/wav.ts`) and stored by Rust (`speech/clips.rs`, `save_clip` with a raw binary IPC body) in `<app data>/recordings`.
- Audio import with any file the WebView can decode (WAV, MP3, M4A/AAC, Ogg/Opus, WebM), converted to the same 16 kHz WAV. The owner's WhatsApp `.ogg` (Opus) imported fine: 46.1 s.
- Clip list with playback (lazy "Load player"), selection and deletion. Clips persist on disk and are listed again after a reload (`list_clips`, `read_clip`, `delete_clip`, all restricted to the store directory).
- `speech/metrics.rs`: word-level alignment, WER, CER, substitution/deletion/insertion counts and a diff, exposed as `compare_texts`. Basic normalisation only (case, punctuation, hyphens, apostrophes); digits and decimals are kept, so changed dosages still count as errors. Critical-error flags (numbers, units, negations, drugs) are M5.
- Engine comparison panel (`ComparePanel`): runs the selected clip through the chosen installed models one after the other (sequential on purpose, parallel runs would distort timing) and shows decoding, load time, inference time, RTF, relative inference time, WER/CER against an optional reference (or the word-level disagreement with a chosen baseline row when there is no reference), plus a diff view. The panel states that one recording proves nothing and never ranks engines.
- UI split into components (`ClipPanel`, `ComparePanel`, `DiffView`); vitest added (3 tests).

**Verified (this machine, in the real Tauri window driven through the WebView2 DevTools protocol)**
- 33 Rust tests and 3 TypeScript tests pass; typecheck and production build pass.
- Microphone: WebView2 shows a native permission prompt ("http://localhost:1430 wants to use your microphones", Block / Allow). For my tests I pre-granted it through the debugging protocol (a test harness action, not something the app does). The default device (Realtek microphone array) was recorded; the saved WAV files are real signals (16 kHz, mono, 16-bit, RMS 0.012 to 0.038).
- Recording duration accuracy: the first version lost about 0.3 s at the end (6.0 s held, 5.72 s saved). Fixed by flushing the worklet buffer and waiting for pending messages before closing; now 5.92 s for 6.0 s held (about 1.5 % short, ~0.1 s; residual cause not investigated).
- Import of the owner's Ogg/Opus voice note decoded by the WebView on Windows (46.1 s). On macOS (WKWebView) Ogg/Opus support is NOT VERIFIED.
- Comparison of three models on the public sample `fr.wav` with its known reference sentence (22 words): Whisper tiny via sherpa-onnx WER 9.1 % (2 substitutions: vous to vos, plutôt to puto), CER 3.5 %; Canary 180M int8 0 %; whisper.cpp tiny (5 beams) 0 %. Single clip, not a ranking. Inference in that run: 479 ms (sherpa Whisper tiny), 767 ms (Canary), 751 ms (whisper.cpp tiny).
- Deleting clips from the UI removes the files (the store went from 4 files to 0), including the converted copy of the owner's voice note. Files in `wav/` untouched.

**Failed / surprises**
- My first UI test attempt hung because the native permission prompt is a separate WebView2 window and my script attached to it instead of the app page. Scripts now select the page by URL. This also showed what the owner will actually see on first recording.
- The clip list lived only in memory: after a reload the saved WAV files became invisible and undeletable (a privacy defect, I-027). Fixed by listing the store at startup.
- `vitest` first picked up a test file from the vendored whisper.cpp sources; its include is now limited to `src/**/*.test.ts`.
- A large multi-edit shell command was rejected by the shell tool again; finished with single edits.

**Not verified**
- Whether the microphone permission is remembered across app restarts, and how the prompt behaves in a release build where the origin is `http://tauri.localhost` (I-024).
- Speaking into the microphone and checking transcription quality (only ambient audio was recorded in my tests; the owner should try with speech).
- Listening to a freshly recorded clip by ear; everything on macOS; other microphones (USB devices exist but were not tried).
- Streaming/real-time latency (the engines here run offline on whole clips; no streaming mode is wired).

**Next**: M5 — benchmark dataset format, reproducible runner, WER/CER with critical-error flags, RTF, memory, repetitions.

### M4 — owner manual tests (2026-10-07, informal, owner-reported; not benchmark data)

- Microphone: Windows asks for permission and recording works (I-024 partly answered: prompt appears and capture works; persistence after restart and release build still untested).
- Recording quality "mediocre", probably because the microphone saturates (owner's hypothesis, plausible but unmeasured). Added a level/clipping indicator after each recording (peak %, share of clipped samples, warnings for clipping or very quiet input) and a unit-tested `signalStats` helper (I-028).
- French speech, own voice, short clips: sherpa-onnx + Whisper tiny replaces some words ("not very precise"); whisper.cpp + Whisper tiny also replaces some words; sherpa-onnx + Canary "much more precise" and even recognises badly pronounced words.
- Import of WAV, Ogg and MP3 works.
- Long English clip (the 1969 "one small step" speech, MP3, poor audio quality, source and license of this file unknown, not part of the repository): whisper.cpp + Whisper tiny gets almost to the end but with repetitions at the end; sherpa-onnx + Canary is the best of the tested ones but its transcript is truncated. Consistent with the earlier private-recording findings (I-018 repetition loops in Whisper tiny, I-019 Canary dropping the ending). Not measured: no reference transcript was used.
- Only the models installed so far were tested (Whisper tiny in both runtimes, Canary int8). No English recording by the owner yet.

---

## 2026-10-07 — M5a — Dataset format, scripts, spoken-number folding, critical-error detector

M5 is split in three parts. This is part a (data and scoring foundations). Part b: dataset recorder in the app (read a sentence, record, save the sample with its reference). Part c: reproducible runner (models x decodings x samples x repetitions, with memory and system info, JSON/CSV results) and chunking experiments (I-029).

**Owner answers used**: priority order 1) short general sentences (questions and statements), 2) medical/administrative/legal vocabulary, 3) IT vocabulary (D-025).

**Done**
- `benchmark/` data folder and `benchmark/README.md` (format, privacy rules, recording guidelines). Layout: `scripts/` (sentences, committed), `samples/` (metadata + reference per sample, committed), `samples-private/` (git-ignored, for third-party voices), `audio/` (git-ignored), `results/`.
- 95 reading items in `benchmark/scripts/`: French general 24 (12 questions, 12 statements), French medical 10 / administrative 7 / legal 7, French with English technical terms 20, English general 16, English technical 8, long dictations 3 (2 French, 1 English). Invented sentences, not medical or legal advice. Written from scratch, no external text.
- `speech/dataset.rs`: loads and validates samples (language, domain, utterance type, source kind and licence, duplicate ids, empty reference, audio presence, duration measured from the WAV, third-party private material must live in `samples-private/` and cannot be committable). Adding samples needs no code change.
- `speech/numbers.rs`: spoken numbers folded to digits in French and English up to 999 999 ("quinze heures trente" = "15 heures 30", "soixante dix" = 70, "quatre vingt dix" = 90, "cinq cents" = 500, "one hundred and five" = 105), "pour cent" to "%", unit spellings unified ("milligrammes" = "mg"). "un/une/one" stay words. Applied to both reference and hypothesis in scoring; different values stay different (D-022 updated).
- `speech/critical.rs`: critical-error flags independent of WER: number changed/lost/added, unit changed, negation dropped/added, weekday/month changed, expected key term missing (with a "probable misspelling" hint from character similarity; drug and name misses are critical, others warnings). French elisions are split for term search ("l'amoxicilline").
- A unit test proves the key point: a 500 mg to 50 mg change gives a WER under 10 % but a critical flag.

**Verified**
- 56 Rust unit tests pass, including: all 95 scripts load, have valid fields, unique ids, and a transcript equal to the reference raises zero flags (guards against a key term that is not literally in its sentence).
- Git state read-only: M4 was committed by the owner on `milestone/m4-audio-compare` (merges of M0-M3 into main done); these M5a changes are uncommitted on top.

**Failed / surprises**
- First version of the key-term check missed "amoxicilline" inside "l'amoxicilline" (French elision glued the words): found by the unit test, fixed by splitting at apostrophes for term search.
- While writing the detector tests I produced one nonsensical assertion line (a runaway generation); caught on reading the file before running, replaced with a real assertion. Noted because it shows why generated test code must be read.
- Number-word handling is a heuristic: compound forms beyond the supported grammar are left untouched, which can only add a reported difference, not hide one. Documented in the module.
- Shell tool rejected several large inline scripts again; used files and single edits.

**Not verified**
- Detector precision/recall on real ASR output (will be measured when the benchmark runs: every flag keeps expected/found text so false alarms can be counted by hand).
- Whether the sentences are natural to read aloud (owner feedback after recording).
- Swiss-French wording differences (septante, huitante, nonante) are NOT folded: a Swiss speaker saying "septante" gets "70" only if the ASR writes digits; the folder does not know these words. Add them if Swiss clips show it matters (I-030).

**Next**: M5b, the in-app dataset recorder (shows the next sentence, records, saves the sample JSON and WAV, re-record button, progress per category), then M5c, the runner.

---

## 2026-10-07 — M5b — In-app dataset recorder

**Git incident first (see ISSUES I-031)**: pushing the new M5 branch failed with GitHub HTTP 500 several times. Bisect by the owner showed that creating any new ref pointing at the M4 commit failed while creating the branch from `main` in the web UI worked; the later push was then a normal fast-forward (`f3917f6..99fb7b6`). M5a is therefore on GitHub (commits d7f2755 and 99fb7b6).

**Done**
- `dataset::create_sample` / `delete_sample` / `sanitize_id` (Rust): turn a recorded clip into a sample (WAV in `benchmark/audio/`, JSON in `samples/` or `samples-private/`), with the reference taken from the script text, never typed by hand. Same speaker + same sentence replaces the previous sample. Third-party voices are forced into `samples-private/` and never committable. Ids are sanitised (no path tricks), bad WAV payloads are rejected and leave no file behind.
- Commands: `list_scripts`, `list_samples` (with validation issues), `create_sample`, `delete_sample`, `read_sample_audio`.
- Scripts are now ordered by the owner's priority (general sentences, medical, administrative, legal, French with English terms, English, long dictations last).
- UI `DatasetRecorder`: speaker form (id, honest profile, optional accent, own voice or someone else's), category selector with "recorded x/y" progress, one large sentence at a time with its key terms, Record/Stop, level check (clipping or too quiet warning), preview player, Save and continue / Redo, Previous/Next, "already recorded" badge, listen to or delete a saved recording, list of dataset validation issues. The speaker form is remembered in the WebView storage (a convenience only).

**Verified (real Tauri window driven through the WebView2 DevTools protocol; microphone permission pre-granted by the test harness; ambient audio only)**
- 61 Rust tests (5 new: id sanitising, create + reload roundtrip incl. replacement, forced private placement, rejection of bad input, delete with path-trick rejection, plus script ordering) and typecheck pass.
- With a test speaker `e2etest`: category list shows the counts, the first sentence is `fr-gen-q-01`; recording, preview ("Level OK, peak 19 %, 2.1 s"), Save created exactly one WAV in `audio/` and one JSON in `samples/` with the right reference, `own-recording`, `committable: false`, measured duration; the UI advanced to the next sentence; "already recorded" badge, listening and deletion worked and removed both files. The same flow with "someone else's voice" wrote into `samples-private/` with `third-party-private`. All test files were removed afterwards (checked: `benchmark/` contains only the README and scripts, and the clip store is empty).

**Failed / surprises**
- "All categories" initially started on the long dictation because script files load alphabetically; fixed by an explicit priority order and defaulting the UI to the first category.
- My UI driver printed mangled letters ("que tion") because of a quoting problem in the test script; the screenshot showed the app itself is correct.
- The displayed benchmark path shows `src-tauri\..\benchmark` (cosmetic, I-032).
- Several large inline shell scripts were rejected again; used script files.

**Not verified**
- Real speech: only ambient audio was recorded in my tests. The owner should record a few sentences and say whether the sentences feel natural to read.
- Reading Swiss-French or other speakers through the same form (needs the speakers; the form supports it).
- Dictation items (long) in the recorder; the Opus/WebView import path is not involved here.

**Next**: the owner records the dataset (suggested: `fr-general` first, 24 sentences, about 5 minutes). Then M5c: the reproducible runner.


---

## 2026-10-07 — M5c — Reproducible benchmark runner and first full benchmark

**Owner action first**: the owner recorded all 95 sentences with the dataset recorder (own voice, speaker `owner`, standard French, 9.1 minutes). M5b is committed; `benchmark/samples/` (JSON metadata, no audio) and `README.md` were still uncommitted.

**Done**
- `speech/probe.rs` (sampled peak memory and CPU time of a process; a dropped sampler stops its thread), `speech/benchmark.rs` (system info, audio quality analysis, runner, per-run records, summaries, hard/suspect sample detection, rescoring, CSV/Markdown/JSONL outputs), CLI `examples/bench.rs` with `check`, `run`, `run-one` (internal), `summarize`, `rescore`. One child process per model configuration (D-029). `scripts/bootstrap_ci.py` for confidence intervals.
- Result fields added to every transcription: peak memory and CPU time (UI contract and runner).
- Scoring versioning: `SCORING_VERSION = 2`; old result files can be re-scored from the stored transcripts without running any engine (`bench rescore`); the previous scores are kept as `runs.scoring-v1.jsonl`.

**Dataset check (`bench check`, VERIFIED)**: 95 samples, 0 validation issues, 16 kHz mono; no clipping (0 files with more than 0.1 % of samples at full scale, 0 "hot" files), peaks 0.75 to 0.994 (most files peak at 0.993, i.e. a few isolated peaks, not sustained saturation); one warning: `en-it-05-owner` (slow reading, 1.2 words/s).

**First full benchmark** (`20261007-201137-full-owner-reps1`; Intel Core 7 150U, 12 logical cores, 23.6 GB, Windows 11, CPU only, release build, 4 threads per engine, 95 samples, 1 repetition, all 9 configurations, 0 failures; one speaker, 1 100 reference words, so this is NOT a general claim). Scoring v2. WER micro with a 95 % bootstrap interval over sentences:

| Configuration | WER | 95 % CI | Critical samples | RTF median | Inference median | Cold load | Peak memory |
|---|---|---|---|---|---|---|---|
| sherpa-onnx Parakeet TDT 0.6B v3 int8 | 2.3 % | 1.4–3.4 | 4 | 0.13 | 0.63 s | 2.5 s | 1.9 GB |
| whisper.cpp small q5_1, 5 beams | 3.9 % | 2.5–5.4 | 7 | 1.85 | 8.3 s | 0.27 s | 0.53 GB |
| whisper.cpp small q5_1, greedy | 4.5 % | 3.0–6.1 | 7 | 1.56 | 7.4 s | 0.27 s | 0.41 GB |
| sherpa-onnx Canary 180M flash int8 | 5.9 % (3.0 % without one garbage output) | 2.0–13.1 | 3 | 0.13 | 0.66 s | 1.2 s | 1.1 GB |
| whisper.cpp base q5_1, 5 beams | 8.4 % | 6.3–10.9 | 12 | 0.37 | 1.8 s | 0.10 s | 0.27 GB |
| whisper.cpp base q5_1, greedy | 11.4 % | 8.8–14.2 | 15 | 0.36 | 1.7 s | 0.10 s | 0.22 GB |
| whisper.cpp tiny, 5 beams | 16.1 % | 12.8–19.9 | 13 | 0.18 | 0.86 s | 0.11 s | 0.24 GB |
| whisper.cpp tiny, greedy | 19.8 % | 15.8–23.9 | 18 | 0.15 | 0.71 s | 0.11 s | 0.21 GB |
| sherpa-onnx Whisper tiny (greedy) | 25.4 % | 19.8–31.9 | 19 | 0.11 | 0.57 s | 0.52 s | 0.89 GB |

What the data supports (and no more):
- Parakeet has the lowest WER and is clearly better than every Whisper configuration (paired bootstrap, interval of the difference excludes 0; against whisper.cpp small it is borderline: lower bound +0.1 point). It is NOT statistically distinguishable from Canary once the one garbage Canary output is set aside (diff +0.8 points, interval -0.6 to +2.3).
- Both sherpa-onnx NeMo models are about 14 times faster than real time on this CPU. whisper.cpp small is SLOWER than real time here (RTF 1.6 to 1.9; median 7 to 8 s for a 5-second sentence, p95 about 14 s), i.e. unusable for interactive voice queries on this machine without acceleration. whisper.cpp base/tiny are fast but much less accurate.
- Model size matters within Whisper: tiny 16 to 20 %, base 8 to 11 %, small 4 to 5 %. Beam search (5) beats greedy by 0.6 to 3.3 points in every size, at 5 to 30 % more time.
- The same Whisper tiny weights score 16 % in whisper.cpp (5 beams) and 25 % in sherpa-onnx: runtime/decoding behaviour matters, not only the weights (I-023, I-018).
- Dictations of about 30 s were transcribed without truncation by all models (output length 0.87 to 1.11 times the reference); the truncation seen earlier concerned a 45 s clip, so the limit lies somewhere between and still needs chunking tests (I-029).
- Language and vocabulary (Parakeet): French 2.1 %, English 2.8 %, medical 1.5 %, administrative 1.5 %, legal 1.5 %, IT 2.7 %. Canary: French 3.0 %, English 15.1 % (one garbage output; otherwise small), IT 15.5 % (same garbage output plus tech terms).
- Critical errors exist in EVERY configuration, including the best: all models misspell "amoxicilline" (Parakeet "amoxycilline", Canary "amoxiciline", Whisper small "amoxiciline"), "ibuprofène" is garbled by Canary and Whisper small, Parakeet turned "10 jours" into "1 jours". A low WER does not make the output safe for clinical vocabulary: a drug-name dictionary or hotword correction step is required (to be tested, e.g. sherpa-onnx hotwords on the transducer).

**Scoring defects found and fixed during this work (honest list, all in my own code)**
1. "ten thirty" was added up to 40 (a time is two numbers); number words now follow a grammar.
2. ASR formats "1030", "12,000" were counted as errors against "10 30", "12000"; digit groups are canonical now.
3. "14h", "9h30", "50%" glued forms were not split, so the number looked "lost"; they are split now.
4. French "dix-huit" was read 10 + 8 = "108"; 17, 18, 19 are handled; spoken decimals ("deux virgule cinq") and the unit "h" ("heures") added.
5. The "Flags" column mixed critical flags and warnings; it is labelled "incl. warnings" and "Critical samples" counts critical severity only.
Re-scoring the same transcripts lowered every WER (for example Parakeet 3.6 % to 2.3 %, Whisper small 5.1 % to 3.9 %). Lesson recorded in D-022/D-030: validate scoring rules on real outputs before trusting any number.

**Experiment: is sherpa-onnx Whisper's repetition caused by my `tail_paddings` setting? NO (VERIFIED).** On 40 French and English general sentences: -1 (my default), 0 and 1000 give identical results (WER 26.8 %, 3 runaway repetition outputs); 50 and 300 are much worse (WER 127 % and 74 %). The default is as good as any tested value, so the loops belong to the engine/model, not to my configuration. Set `SPEECHLAB_SHERPA_WHISPER_TAIL_PADDINGS` to reproduce.

**Failed / surprises**
- All nine configurations hear "port 543" where the script says "port 443" (`en-it-05`): the reading was probably not "443" (I-033). Until the owner listens, that sample is suspect and is reported separately.
- Canary produced a string of single letters ("O P O N T H O R E N T...", WER 517 %) on that slow, unusual recording: a runaway failure mode that needs a guard in any product (I-035).
- The first session was interrupted while the benchmark was running; the benchmark process survived and finished (the monitors were lost, the work was not).
- A shell tool limitation again forced script files instead of inline scripts.

**Not verified**
- Anything about other speakers, accents (Swiss or Maghreb clips are not in the dataset yet), microphones or noise.
- Timing variance (1 repetition only; I-012): a 3-repetition timing study is still to do.
- Memory figures are sampled lower bounds; for sherpa-onnx they cover the whole benchmark child process.
- Long-audio behaviour beyond 30 s (chunking/VAD) and hotword biasing for drug names.
- whisper.cpp turbo and any GPU/accelerated build.

**Next**: owner listens to `en-it-05` and decides; commit M5c; then (a) timing study with repetitions, (b) chunking/VAD tests for long audio, (c) hotword/drug-name correction test, (d) import the Swiss-French and Maghreb-accent clips as private long samples with references.


---

## 2026-10-07 — M5c — The first benchmark was run on a busy machine; clean re-run prepared

**What happened**: the owner reported having started a heavy task on the computer while the first full benchmark was running. The run (`20261007-201137-full-owner-reps1`) is therefore valid for ACCURACY but its SPEED, MEMORY and BUSY-CORE figures are unreliable. The numbers quoted in the M5c entry above for RTF, inference time, cold load and peak memory must be read as "measured under load" until the clean re-run replaces them. A `NOTE.md` was added in that results folder; nothing else in it was changed.

**Why accuracy is safe (expected, to be verified)**: the engines are deterministic for a given input and thread count, so the transcripts, WER and flags should be identical in a clean run. The re-run will compare them sample by sample and report any difference. Speed figures may change a lot (probably faster and less variable, but that is a guess to be measured).

**New safeguards (VERIFIED by running the tool)**
- `bench run` now performs a quiet-machine check before starting: it observes CPU use for 8 seconds and the power source, lists the busiest processes, and REFUSES to run when the machine averages more than 15 % CPU or runs on battery, unless `--force` is passed. The measurements and a `forcedDespitePreflight` flag are stored in `config.json`; `system.json` now also records the Windows power plan and AC/battery state.
- Tried right now: preflight FAILED (CPU 35 % average; Windows counters show 16 to 52 % with high kernel time). The visible user processes explain only about 5 %; Docker Desktop and the WSL virtual machine (`vmmemWSL`, 3.4 GB resident, Docker Desktop with a large accumulated CPU time) are active and are the likely cause, as CPU used inside the WSL VM is not attributed to a process. Avast also runs services. Nothing was stopped by the assistant: Docker and WSL belong to the owner's other work.

**Next**: the owner stops the heavy task (for example quit Docker Desktop and run `wsl --shutdown`), keeps the charger plugged in and leaves the computer alone for about 45 minutes; the assistant re-runs the preflight and launches the clean run with the same settings (all installed models, 95 samples, 1 repetition, label `full-owner-reps1-clean`), then compares accuracy with the first run (determinism check) and replaces the speed figures. An optional timing study with 3 repetitions on a subset follows.

---

## 2026-10-07 — Hand-over documentation for new sessions

**Done**
- `docs/HANDOFF.md`: one document with the project summary, the rules most often broken, the state of every milestone, the repository map, the environment traps (port 1430, antivirus TLS, quiet machine, interrupted sessions, GitHub 500, shell quoting, UI automation), all commands, the prioritised backlog and the session protocol.
- `docs/prompts/`: `01-clean-benchmark-rerun.md` (complete prompt for the immediate next task), `next-tasks.md` (common preamble plus tasks T2 timing study, T3 long-audio VAD chunking, T4 drug-name handling, T5 private accent clips, T6 TTS, T7 packaging, T8 final report) and a README explaining how to use them.
- `AGENTS.md` start/end checklist now points to HANDOFF and the prompts and has a section on parallel sessions and long jobs.
- Tools needed by the procedure: `bench preflight` (quiet-machine check on its own, exit code 0/2) and `scripts/compare_runs.py` (transcript determinism and speed comparison of two runs; tested by comparing a run with itself: 855 of 855 identical, as it must).

**Verified**: `bench preflight` ran and reported NOT QUIET (30 % CPU average) while Docker Desktop and the WSL virtual machine `docker-desktop` were still running after the owner had closed their other applications; no process of the owner was stopped.

**Not verified**: that a fresh session following prompt 01 completes the clean re-run without help (it is written so; the first real use will show gaps: record them here).

**Next**: the owner closes Docker Desktop and runs `wsl --shutdown`, then opens a new chat tab and pastes `docs/prompts/01-clean-benchmark-rerun.md`.

---

## 2026-10-07 — M5c — Clean re-run attempted: blocked by the quiet-machine check

**Done**
- Read-only checks: `git status` clean on `milestone/m5-benchmark`, no `bench.exe` or `whisper-cli.exe` running. `cargo build --release --example bench` up to date (nothing to compile).
- `bench preflight` run 10 times in a row over several minutes: exit code 2 every time. Average CPU 15.6 %, 16 %, 18 %, 24 %, 24 %, 25 %, 25 %, 27 %, 29 %, 31 % (limit 15 %). AC power: yes. Power plan: "Utilisation normale" (balanced).

**Diagnosis (VERIFIED by observation)**
- Docker Desktop and the WSL virtual machine are NOT the cause this time: `wsl -l -v` shows `docker-desktop` Stopped, there is no `vmmemWSL` and no Docker process.
- Visible user processes (code editor, shell) add up to about 2 % of the CPU.
- The per-process performance counters show a Windows service host running the Windows Firewall and Base Filtering Engine (`BFE`, `mpssvc`) at 86 to 96 % of one core (about 8 % of the 12 logical cores) in every sample, plus the Avast Firewall service (`afwServ`, 0 to 24 %) and the Avast service (`AvastSvc`, up to 38 %). Windows reports 29 to 36 % privileged (kernel) time, which fits network-filter activity. Cause of the firewall load (rule churn, a firewall conflict, a scan) is NOT determined; admin rights would be needed to look further.
- Nothing was stopped, killed or changed: these are system and security services of the owner.

**Not done**: the clean benchmark was NOT started (no `--force`, by rule). The first run's speed figures are therefore still unreliable (I-039 stays open).

**Next**: the owner decides how to bring the machine under 15 % CPU (see the message in the chat), then `bench preflight` is repeated until exit code 0 and prompt `docs/prompts/01-clean-benchmark-rerun.md` is continued from step 4.

---

## 2026-10-07 — M5c — Clean re-run of the full benchmark (realistic background load)

**Done**
- Quiet-machine limit relaxed from 15 % to 30 % background CPU at the owner's request (D-034): on this laptop the background load never fell below 16 % even with the owner's applications closed, Docker/WSL stopped and the security software's shields switched off (its services kept running). Reason given by the owner: an application on a client machine will also share the CPU with other processes, so a realistic load is the more useful test. Code change: constant `MAX_BACKGROUND_CPU_PERCENT` in `src-tauri/examples/bench.rs`.
- Run `20261007-215116-full-owner-reps1-clean`: same settings as the first run (all 6 installed models, both whisper.cpp decodings, 9 configurations, 95 samples, 1 repetition, 4 threads per engine, release build, isolated child process per configuration), launched with `bench.exe run --reps 1 --label full-owner-reps1-clean`. Nothing else was run on the machine while it ran (only light log reads). 0 failures.
- Comparison with the first run (`scripts/compare_runs.py`), confidence intervals (`scripts/bootstrap_ci.py`, with and without `en-it-05-owner`).

**Conditions (VERIFIED, from `config.json` and `system.json`)**: `forcedDespitePreflight` = false; preflight average CPU **19.4 %** over 8 s (the reading just before had been 21.7 to 25 %); AC power; Windows power plan "Utilisation normale" (balanced); no Docker/WSL process. The background load came mostly from the Windows firewall engine, the DNS client, the capability-access service and the security software's services. This is "a realistic background load of about 20 %", NOT an idle machine. The load during the 45-minute run was not logged continuously (NOT VERIFIED beyond the start reading).

**Determinism (VERIFIED)**: 855 of 855 transcripts (9 configurations x 95 samples) are identical between the busy run and this run. Consequently WER, CER, critical flags and every accuracy table of the first run stand unchanged; accuracy does not depend on machine load here (for this engine and thread configuration on this machine). No transcript difference needed investigating.

**NEW speed figures (replace the unreliable ones of the first run; measured at about 20 % background CPU, one speaker, 95 sentences, 1 repetition; "old" = disturbed first run)**

| Configuration | WER | RTF median (old -> new) | Inference median ms (old -> new) | Inference p95 ms (old -> new) | Cold load ms (old -> new) | Peak memory MB (old -> new) | Busy cores (old -> new) |
|---|---|---|---|---|---|---|---|
| sherpa-onnx Parakeet TDT v3 int8 | 2.3 % | 0.130 -> 0.114 | 630 -> 551 | 940 -> 785 | 2463 -> 1956 | 1875 -> 1875 | 9.0 -> 9.4 |
| whisper.cpp small q5_1, 5 beams | 3.9 % | 1.851 -> 1.236 | 8283 -> 5881 | 14556 -> 6837 | 267 -> 199 | 532 -> 533 | 3.8 -> 3.9 |
| whisper.cpp small q5_1, greedy | 4.5 % | 1.563 -> 1.117 | 7358 -> 5403 | 13245 -> 6070 | 269 -> 199 | 411 -> 410 | 3.8 -> 3.9 |
| sherpa-onnx Canary 180M flash int8 | 5.9 % | 0.134 -> 0.111 | 659 -> 536 | 1507 -> 903 | 1238 -> 1147 | 1088 -> 1087 | 7.4 -> 7.4 |
| whisper.cpp base q5_1, 5 beams | 8.4 % | 0.373 -> 0.337 | 1807 -> 1626 | 2380 -> 1910 | 95 -> 88 | 265 -> 265 | 3.8 -> 3.9 |
| whisper.cpp base q5_1, greedy | 11.4 % | 0.355 -> 0.293 | 1688 -> 1426 | 2319 -> 1626 | 97 -> 88 | 217 -> 216 | 3.8 -> 3.8 |
| whisper.cpp tiny, 5 beams | 16.1 % | 0.177 -> 0.159 | 857 -> 762 | 1087 -> 915 | 110 -> 104 | 244 -> 242 | 3.7 -> 3.7 |
| whisper.cpp tiny, greedy | 19.8 % | 0.149 -> 0.129 | 705 -> 625 | 918 -> 674 | 112 -> 103 | 214 -> 213 | 3.5 -> 3.5 |
| sherpa-onnx Whisper tiny, greedy | 25.4 % | 0.113 -> 0.077 | 566 -> 365 | 863 -> 578 | 523 -> 490 | 894 -> 894 | 7.4 -> 7.4 |

What changed and what did not:
- Memory and busy cores did not depend on the load (equal to within 1 %); they are properties of the engines. Speed improved by 10 to 35 %, and the tail (p95) shrank most: whisper.cpp small p95 fell from 13.2 to 14.6 s to 6.1 to 6.8 s. The disturbed first run overstated the slowness, mostly in the tail.
- The ranking and the conclusions about speed hold: the two NeMo models run at RTF 0.11 (about 9 times faster than real time); whisper.cpp small remains SLOWER than real time on this CPU (RTF 1.12 greedy, 1.24 with 5 beams, median 5.4 to 5.9 s for a sentence of about 5 s) even with the first run's disturbance removed; whisper.cpp base runs about 3 times faster than real time but at 8 to 11 % WER.
- The earlier statement "about 14 times faster than real time" for the NeMo models (RTF 0.13) is superseded by RTF 0.11, about 9 times.
- sherpa-onnx still uses far more cores than requested (7.4 to 9.4 busy cores with 4 threads configured; whisper.cpp 3.5 to 3.9): I-036 is confirmed, not an artefact of the disturbance.
- Confidence intervals are unchanged (same transcripts): Parakeet 2.3 % [1.4, 3.4]; against whisper.cpp small with 5 beams the difference is +1.6 points [+0.1, +3.1], borderline; against Canary +3.6 [-0.3, +10.5] with the garbage output and +0.8 [-0.6, +2.3] without it. With `--exclude en-it-05-owner`: Parakeet 2.2 %, Canary 3.0 %, small 3.7 %. Excluding the suspect sample changes no conclusion.

**Failed or surprises**
- The preflight could not pass at 15 % on this machine (previous entry); at 20 % it still failed (21 %, 25 %, 21 %); 30 % passed. The relaxation is a documented decision (D-034), not a hidden workaround.
- The preflight reading varies by more than 5 points from one call to the next (16 to 31 %), so the background load during the run was probably not constant; the speed spread is larger than on a quiet machine and one repetition cannot quantify it (I-012).
- Peak memory in `summary.md` is the maximum over samples, while `compare_runs.py` shows the median; do not mix the two.

**Not verified**
- Timing variance across repetitions (I-012): only 1 repetition.
- Speed under other loads or on another machine; any other speaker, accent or microphone; GPU or accelerated builds; whisper.cpp turbo.
- Whether the security software's real-time shields were fully off during the run (the owner switched them off; its services were still running).

**Next**: timing study with 3 repetitions on a subset (backlog item 3, `docs/prompts/next-tasks.md` T2) under the same D-034 conditions, logging the background CPU load before and after.

**Identity check (end of entry)**: a search for AI and vendor names found one editor name in the earlier "blocked" entry above and one in the `topProcesses` field of the clean run's `config.json`; both were replaced by a neutral label ("code editor", `code-editor.exe`) in the working copy. The earlier wording is still in the already committed history. The remaining matches are licence attributions of the Whisper model weights (`models-manifest.json`, `docs/M0_FEASIBILITY.md`), which are factual licence data, not self-references. Private audio and transcripts of the owner's recordings are not quoted anywhere in this entry; `summary.md` contains only the printed scripts (the reference sentences), not the audio.


---

## 2026-10-08 — M5d — Timing study with 3 repetitions (I-012)

**Done**
- Run `20261008-035938-timing-3reps`: 9 configurations (all 6 installed models, both whisper.cpp decodings), categories `fr-general` and `en-general` (40 sentences, 2.9 minutes of audio), 3 repetitions each = 1080 runs, 0 failures, release build, 4 threads per engine, isolated child process per configuration. Launched with `bench.exe run --reps 3 --category fr-general,en-general --label timing-3reps`; nothing else ran on the machine (light log reads only). Duration about 45 minutes.
- New helper `scripts/timing_study.py`: per configuration the inference median, p95, min, max, the coefficient of variation (CV = standard deviation / mean) across the 3 repetitions of the same sample, the median per repetition, changing transcripts, memory and busy cores.
- The preflight line of `config.json` listed the code editor by its executable name; replaced by the neutral label `code-editor.exe` in the working copy (same rule as before).

**Conditions (VERIFIED, `config.json`, `system.json`)**: `forcedDespitePreflight` = false; background CPU **19.9 %** over 8 s before the run and **21.5 %** over 8 s right after it (two readings, NOT a continuous log); AC power; Windows power plan "Utilisation normale" (balanced); no Docker/WSL. Same D-034 conditions as the clean full run. One speaker, CPU only, short sentences (about 4.4 s on average).

**Results (VERIFIED)**

Run-to-run variation of the SAME sample inside this run (3 consecutive repetitions):

| Configuration | WER (rep 1) | Inference median ms | p95 ms | min - max ms | RTF median | CV median % | CV worst sample % | Median per repetition (rep 1 / 2 / 3, ms) | Peak mem MB (median) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-onnx Canary 180M flash int8 | 1.5 % | 414 | 528 | 307 - 654 | 0.095 | 2.4 | 9.0 | 416 / 417 / 410 | 761 | 7.4 |
| sherpa-onnx Parakeet TDT v3 int8 | 2.7 % | 440 | 558 | 322 - 620 | 0.099 | 3.5 | 19.1 | 432 / 450 / 439 | 1578 | 10.4 |
| sherpa-onnx Whisper tiny, greedy | 25.9 % | 268 | 400 | 192 - 460 | 0.059 | 2.4 | 16.1 | 268 / 260 / 270 | 669 | 7.5 |
| whisper.cpp small q5_1, 5 beams | 2.7 % | 5697 | 6332 | 5001 - 6724 | 1.318 | 0.7 | 6.4 | 5677 / 5736 / 5686 | 478 | 3.9 |
| whisper.cpp small q5_1, greedy | 3.4 % | 5374 | 5944 | 5184 - 6344 | 1.261 | 1.4 | 7.1 | 5357 / 5374 / 5406 | 360 | 3.9 |
| whisper.cpp base q5_1, 5 beams | 6.4 % | 1356 | 1497 | 1239 - 1585 | 0.313 | 1.2 | 5.9 | 1364 / 1352 / 1350 | 212 | 3.8 |
| whisper.cpp base q5_1, greedy | 7.9 % | 1272 | 1373 | 1188 - 1409 | 0.293 | 0.7 | 6.4 | 1270 / 1279 / 1264 | 167 | 3.8 |
| whisper.cpp tiny, 5 beams | 11.0 % | 608 | 696 | 546 - 748 | 0.143 | 1.2 | 6.6 | 604 / 606 / 618 | 190 | 3.7 |
| whisper.cpp tiny, greedy | 13.1 % | 517 | 549 | 485 - 636 | 0.121 | 0.9 | 7.8 | 516 / 522 / 517 | 163 | 3.5 |

- **Transcripts are stable across repetitions**: 0 samples with a changing transcript in all 9 configurations (`unstableSamples` = 0), and the 40 first-repetition transcripts of every configuration are identical to the same 40 samples in the clean full run (360 of 360). Third confirmation that accuracy is deterministic.
- **Within-run spread is small**: the median CV is 0.7 to 1.4 % for whisper.cpp and 2.4 to 3.5 % for sherpa-onnx; the worst single sample reaches 6 to 8 % (whisper.cpp) and 9 to 19 % (sherpa-onnx, whose sentences take only 200 to 650 ms, so a few tens of milliseconds weigh more). The per-repetition medians differ by less than 5 % (sherpa-onnx) and less than 1.5 % (whisper.cpp).
- **No cold-versus-warm effect on inference (VERIFIED)**: for sherpa-onnx the model stays loaded between repetitions; repetition 1 is not slower than repetitions 2 and 3 (Canary 416 / 417 / 410 ms). The first inference after loading (first sample) was not slower either (221 to 383 ms, below the medians). The cold cost is the model LOAD only (first record: Canary 1049 ms, Parakeet 1912 ms, Whisper tiny 545 ms; whisper.cpp reloads in every process: 75 to 87 ms for tiny/base, about 200 ms for small, since the file is in the operating system cache).
- **Between-run difference is larger than within-run spread**: on the same 40 samples, the median inference of the clean full run (repetition 1) versus this run was Canary 463 -> 417 ms, Parakeet 517 -> 433 ms, Whisper tiny 325 -> 268 ms, whisper.cpp base 1565 -> 1364 ms (5 beams) and 1431 -> 1270 ms (greedy), tiny 716 -> 605 and 613 -> 516 ms, small 5745 -> 5677 and 5364 -> 5357 ms. That is up to 20 % faster in this run for the short models and nothing for whisper.cpp small. The cause was NOT determined (the background load differed between runs but was not logged continuously; CPU frequency behaviour is a candidate). So a single run cannot give a precision better than about 10 to 20 % for the speed of the small, fast models; the figures of the clean full run should be read with that margin. The CONCLUSIONS (ranking, order of magnitude of the RTF) are unchanged.
- **Speed picture unchanged**: Canary and Parakeet run at RTF 0.10 (about 10 times faster than real time) on these sentences, Whisper tiny on sherpa-onnx at 0.06, whisper.cpp base at 0.29 to 0.31, tiny at 0.12 to 0.14, and whisper.cpp small stays SLOWER than real time (RTF 1.26 greedy, 1.32 with 5 beams, median 5.4 to 5.7 s per sentence of about 4.4 s) even with the variance accounted for: its whole min-max range (5.0 to 6.7 s) lies above the sentence duration. Memory and busy cores agree with the full run; peak memory is lower here only because this table shows the median over short sentences while the full summary shows the maximum over samples.
- **Accuracy on this subset (short sentences only)**: Canary 1.5 % [0.3, 2.8], Parakeet 2.7 %, whisper.cpp small 2.7 % (5 beams) and 3.4 % (greedy); Canary, Parakeet and small with 5 beams are not distinguishable (bootstrap, 40 samples, seed 7); small greedy is clearly worse than Canary here (+1.8 points [+0.3, +3.7]). The full-set picture (Parakeet first, Canary hurt by one garbage output on a long clip) is not contradicted: Canary's failure was on long audio, which this subset does not contain.

**Failed or surprises**
- `loadMs` is recorded on the first sample only for sherpa-onnx, so the per-repetition median load printed by the helper script reads 0; the cold load figures above are the first records' `loadMs`, quoted by hand (I-040, then fixed in the script: it now prints the maximum load).
- The run was NOT quiet in the strict sense (about 20 % background CPU from system services, D-034). It does not show behaviour under heavy load, or thermal throttling over a long session: the repetitions of a sample are consecutive, and the three repetition medians show no drift, but a slow drift across the whole 45-minute run was not tested.

**Not verified**
- Cause of the run-to-run offset (up to 20 % for fast models, none for whisper.cpp small).
- Speed under heavier load, on battery, on another machine, with long audio (this subset has no clip over about 10 s), with another speaker or microphone, or with GPU builds.
- Why sherpa-onnx uses 7 to 10 busy cores for 4 requested threads (I-036).

**Next**: long-audio chunking with a voice-activity detector (backlog item 4; prompt `docs/prompts/02-long-audio-vad-chunking.md`). The owner's check of the suspect sample `en-it-05-owner` (I-033) is still pending.


---

## 2026-10-08 — M5e — Long audio: chunking at silences with a voice-activity detector (I-017, I-018, I-019, I-029, I-035)

**Done**
- New support model `silero-vad` (Silero VAD, 643 854 bytes, MIT, SHA-256 `9e2449e1087496d8d4caba907f23e0bd3f78d91fa552479bb9c23ac09cbb1fd6`) in `src-tauri/models-manifest.json` with a new `role: support` field. `ModelManager::list()` leaves support models out (UI pickers and the default model list of `bench` never see it); `support_models()` and `vad_model_path()` reach it. Downloaded with the owner's approval (curl into a scratch folder to read the size, hash and the upstream LICENSE, then the normal installer `transcribe install silero-vad`, which verified the hash again). The guess "about 2 MB" in the prompt was wrong: 0.6 MB.
- New modules: `speech/chunking.rs` (planner `plan_segments`, quietest-point search, an energy-based speech detector used by the tests, `transcribe_chunked` which works with ANY provider through temporary WAV pieces and summed timings) and `speech/vad.rs` (Silero through the `sherpa-onnx` crate). Design and constants: D-035.
- `bench run --chunking vad` (default off, whole-clip path unchanged) and an experiment-only `--chunk-max-s N`; `runs.jsonl` gains `chunking`, `segments`, `chunkingMs` (old files still load); `transcribe run ... --chunking vad --out file.json` for private clips (writes JSON, prints no text); `scripts/chunking_study.py` (modes `runs`, `same`, `clips`).
- 14 new unit tests on synthetic audio (tones and zeros, no recordings): short clip stays one whole segment and reaches the provider untouched, long clip cut in the pauses without losing speech, continuous speech cut at the quietest point (within 0.15 s of the planted dip), 70 s of continuous tone never exceeds the limit, padding clamped, empty and all-silence input, resampling, text joining, summed timings, cancellation, temporary files removed. `cargo test --lib`: 92 passed (77 before this step). The heuristics of the study script were self-tested on synthetic strings.
- Runs (release build, 9 configurations, repetition 1, `forcedDespitePreflight` = false, AC power; the preflight retried automatically when it read 31 % and 38 %, never forced): `20261008-045952-long-whole` (3 dictations, whole clip; background CPU 19.2 % before the run), `20261008-050308-long-vad` (same, chunked at 25 s; 28.4 %), `20261008-050621-short-vad` (the 92 other samples, chunking on; 14.4 %), plus two experiments with a shorter segment limit, `20261008-054129-long-vad-max15` and `20261008-054529-long-vad-max10`. Private clips were run with `transcribe` into a scratch folder outside the repository.

**Verified (observed here; one speaker, one microphone, CPU only)**
- **Short audio is not touched.** In `short-vad` all 828 records have `chunking` = vad and `segments` = 1, and all 828 transcripts are IDENTICAL to the clean full run (`chunking_study.py same`). By design a clip of at most 25 s never reaches the detector. The whole-clip rerun of the 3 dictations is also identical to the clean full run (27 of 27): a fourth determinism check.
- **sherpa-onnx Whisper cannot take more than 30 s**: the library prints "Only waves less than 30 seconds are supported. We process only the first 30 seconds and discard the remaining data". Confirmed on `fr-dict-01` (35.9 s): the whole-clip output stops at "...transmis à la surveillance" (last sentence lost, WER 31.3 %, 76 of 83 words); chunked, the ending is present (WER 22.9 %, 88 words). This is a hard limit, not a quality issue.
- Dictations, whole clip versus chunked at 25 s (3 clips, 54 to 83 reference words each, 199 words in total, so a difference of one or two words is noise): Parakeet 0.5 % -> 0.5 %; Canary 1.5 % -> 6.5 %; Whisper tiny (sherpa) 20.1 % -> 18.1 %; whisper.cpp small 3.0 % -> 1.5 % (5 beams) and 3.0 % -> 2.5 % (greedy); base 5.5 % -> 7.0 % and 8.5 % -> 7.5 %; tiny 12.1 % -> 15.1 % and 19.6 % -> 20.6 %. At 25 s no repetition loop and no runaway output in either mode on these three clips. The heuristic "ending missing" fired on 3 of 27 whole-clip outputs (sherpa Whisper tiny on `fr-dict-01`: a real truncation; whisper.cpp base greedy and tiny greedy on `fr-dict-02`: garbled endings of weak models, not a dropped ending); chunking cleared the first two.
- **Canary got WORSE with 25 s chunks on `en-dict-01`**: WER 3.7 % -> 22.2 %. The first piece (0.3 to 24.3 s) lost a whole sentence ("The server must not be restarted during business hours") and produced the non-word "unablediscovered"; the same words were decoded correctly inside the whole 29.4 s clip. With a 15 s limit the same three clips give Canary 1.5 % (the whole-clip level) and with 10 s 2.0 %.
- **Shorter pieces are not better for everyone**: Parakeet 0.5 % (25 s) -> 3.0 % (15 s) -> 0.5 % (10 s), noise level on 199 words; whisper.cpp small 1.5 % (25 s) -> 2.5 % (15 s) -> 4.0 % (10 s), and base and tiny also worsen at 10 s (`chunking_study.py runs` lists the samples). Each cut removes context. At 15 s one sherpa Whisper tiny output contains a repetition loop that the 25 s and 10 s runs do not.
- **Private clips without reference** (A: 45.8 s recorded at 11.025 kHz; B: 33.6 s; assumed French; only counts and booleans are quoted, no content): on A, Canary returns 104 words whole and only **52 words in 63 s** (RTF 1.4, a runaway like I-035) chunked at 25 s (cut at 23.9 s), and in both modes the closing word is missing from its end (I-019 persists); Parakeet and whisper.cpp tiny, base, small return 107 to 123 words with the closing word, whole or chunked. sherpa Whisper tiny keeps a repetition loop in BOTH modes (a 3-word group repeated 35 times whole and 28 times chunked; 106 and 137 words): chunking does not remove the loops, it only recovers the ending (the closing word is present only when chunked). On B nothing differs materially (63 to 67 words everywhere, no loop).
- Cost: speech detection takes 0.1 to 0.3 s per clip of 25 to 46 s (`chunkingMs`), small beside inference. Sums of inference time measured back to back (`long-whole` versus `long-vad`; Canary 37.9 -> 25.4 s, Parakeet 14.7 -> 12.1 s, whisper.cpp small 40.6 -> 43.2 s with 5 beams) are inside the between-run offset (below), so no speed claim is made.

**Failed or surprises**
- The first launch of the run script was refused by the preflight (31 % against the 30 % limit); I added an automatic retry on exit code 2 (never `--force`). The `long-vad-max15` series also needed one retry (38 %).
- Speed is not comparable between runs: for the same unchanged code path (clips of 25 s or less go straight to the engine) the median inference time of `short-vad` was 1 to 48 % above the clean run (sherpa Whisper tiny 362 -> 534 ms, Canary 532 -> 716 ms, Parakeet 548 -> 674 ms, whisper.cpp small greedy 5402 -> 6062 ms) although the preflight load was lower (14.4 % versus 19.4 %). This is the unexplained between-run offset of I-012, larger here than in the timing study.
- The editor name appeared in `config.json` (`topProcesses`) of every new run folder; replaced by `code-editor.exe` in the working copy, as before.
- Linear-interpolation resampling is used for clips that are not at 16 kHz (clip A); it was not compared with a better resampler.

**Not verified**
- Anything beyond one speaker, one microphone, 3 reference dictations and 2 private clips without reference. Statistical power is very low (one dropped sentence moves a WER by 20 points); no confidence interval was computed on 3 clips.
- Why Canary loses a sentence inside a 24 s piece but not inside a 29 s clip, and why it ran 63 s on private clip A (not isolated: the per-piece outputs were not inspected, to keep the clip private). Only the limits 25, 15 and 10 s were tried; the VAD parameters were not tuned.
- Dictations longer than 46 s, noisy audio or several speakers, and dataset audio at a sample rate other than 16 kHz.
- Whether chunking helps whisper.cpp, which windows long audio itself (no loss and no loop seen here, on 3 clips only).

**Next**: drug-name and key-term handling (backlog item 5; prompt `docs/prompts/03-drug-name-handling.md`). Question for the owner: does the product need dictations over 30 s? If yes, the engine choice for long clips matters (Parakeet and whisper.cpp behaved on every clip; Canary and sherpa Whisper tiny did not).

**Identity check (end of entry)**: the `config.json` files of the five new run folders were neutralised. No private content (names, transcripts) was written; the private clips are identified only as A and B.


---

## 2026-10-08 — T4 — Drug-name and key-term handling (hotwords, initial prompt, dictionary post-correction; D-036)

**Done**
- New code, everything OFF by default (earlier results stay comparable): `TranscribeRequest.vocabulary` (engine-neutral list of terms; empty = no biasing); whisper.cpp turns it into `--prompt "term1, term2."`; sherpa-onnx NeMo transducer (Parakeet) can run modified beam search with per-stream hotwords (`SherpaOnnxProvider::with_hotwords(score)`); `speech/postcorrect.rs` (dictionary correction, three presets strict/medium/loose, unit-tested on synthetic strings); `speech/termstudy.rs` (key-term outcome per configuration, and word-level FIXES versus FALSE CORRECTIONS between a baseline run and a variant run); `bench run --vocab-dir DIR --hotwords-score S`, `bench termstudy --dir A [--against B] [--category ...]`, `bench postcorrect --dir A --vocab-dir DIR --preset P` (works on stored transcripts, no engine run); `examples/hotwords_probe.rs` (the isolated experiment of the first finding below). `RunRecord` gains `biasing`, `postCorrection`, `rawText` (old files still load).
- Vocabulary files `benchmark/vocab/fr.txt` (40 dataset key terms plus 16 common drug names added as distractors) and `en.txt` (10 dataset key terms). The key terms come from the test sentences themselves, so every number below is an UPPER BOUND on what a vocabulary known in advance can do.
- Runs (release build, 1 repetition, all 95 samples, `forcedDespitePreflight` = false, AC power, preflight 16, 19, 16 and 11 % CPU): `20261008-065534-t4-parakeet-beam-control`, `...065715-t4-parakeet-hotwords-1.5`, `...065852-t4-parakeet-hotwords-3.0`, `...070031-t4-whisper-prompt` (tiny, base, small, 5 beams and greedy). Offline (no engine): `...-t4-postcorrect-{strict,medium,loose}-on-clean` applied to the clean full run, plus `...-t4-prompt-plus-postcorrect-strict` and `...-t4-hotwords1.5-plus-postcorrect-strict`. Each folder has `termstudy-vs-clean-run.md` (all 95 samples; use `bench termstudy --category` for subsets). The baseline is always the clean full run `20261007-215116-full-owner-reps1-clean`.
- `cargo test --lib`: 118 passed (92 before; 26 new: post-correction, key-term study, prompt building, surrogate vocabulary). `pnpm typecheck` passes.

**Verified (observed here; one speaker, one microphone, CPU only)**
- **Baseline.** Only 6 drug-name occurrences exist in the public dataset (5 in `fr-medical`, 1 in `fr-dictation`), 10 `term`, 34 `tech`, 6 `legal`: the drug statistics rest on 6 occurrences per configuration, so one fixed word moves a result by 17 points. Drug names found (clean run, all 95 samples): Parakeet 4/6, Canary 4/6, whisper.cpp small 3/6 (both decodings), base 2 to 3/6, tiny 1 to 2/6, sherpa Whisper tiny 1/6. Many misses are one or two letters off ("amoxiciline", "amoxycilline", "ibuprophène"), some are far ("libby profene", "antique ouagulant").
- **Hotwords with the NeMo transducer (Parakeet) work in this library version, but not out of the box.** The path has its own hotword initialisation (`offline-recognizer-transducer-nemo-impl.h`, per-stream and per-recognizer lists both work). The model download has no SentencePiece vocabulary file: with the default character unit every hotword fails to encode ("Cannot find ID for token"), and with `bpe` the library refuses to start without a `bpe.vocab`. A surrogate `bpe.vocab` derived from `tokens.txt` (score = minus the token id) made it work. The real scores are unknown, so the cut of a hotword into pieces may differ from the model's own; this is a limit of the experiment.
- **Beam-search control.** Parakeet with modified beam search and NO vocabulary: same WER (2.3 %), 0 fixed and 0 broken words against greedy, 92 of 95 transcripts identical. So what follows comes from the hotwords, not from the beam.
- **Parakeet hotwords, score 1.5** (all 95): drug names 4/6 -> 5/6, WER 2.3 -> 2.7 %, 1 word fixed, 4 words broken (3 outside key terms, all in the 36 s dictation: "il ne signale" lost, "artérielle" lost). One term went missing (10/10 -> 9/10). The fixed drug costs the elision: "d'amoxicilline" becomes "d amoxicilline" (the hotword starts a new word), which the scoring counts as an error on that word. On the three target categories: drug 4/5 -> 5/5, WER 2.9 -> 3.1 %, 0 fixed, 0 broken words outside the drug.
- **Parakeet hotwords, score 3.0**: drug 6/6, tech 31/34 -> 33/34, but WER 2.3 -> 5.1 %, 30 words broken (29 outside key terms), critical samples 4 -> 7, and one runaway (`en-gen-q-05`, WER 100 %). The engine inserts vocabulary words into unrelated English sentences ("postgresql", "api", "rest" in place of "is", "the", "at") and drops words. A score that is too high is a regression: say so.
- **whisper.cpp initial prompt** (the vocabulary as a comma list), three target categories: small, 5 beams: WER 6.5 -> 3.9 %, tech terms 26/32 -> 31/32, term 3/4 -> 4/4, drug 3/5 -> 3/5 (no gain), 1 word broken; small, greedy: 7.5 -> 5.7 %, drug 3/5 -> 2/5 (one drug LOST), 7 words broken; base: WER 12.7 -> 10.4 % (5 beams) and 16.4 -> 10.6 % (greedy), 9 to 12 words broken; tiny: no WER gain (22.6 -> 22.9 %), critical samples 7 -> 10, 28 to 32 words broken. All 95 samples: small 5 beams 3.9 -> 3.3 % (11 fixed, 7 broken), small greedy 4.5 -> 3.8 % (16 / 11), base 8.4 -> 8.3 % and 11.4 -> 9.8 %, tiny 16.1 -> 17.1 % and 19.8 -> 20.0 % (35 to 39 fixed, 51 to 54 broken). On the six categories without key terms the prompt makes small, base and tiny slightly worse (2.4 -> 3.0 %, 6.0 -> 7.2 %, 12.4 -> 13.8 %, 5 beams), with broken words such as "jeudi" -> "jedi", "veuillez" -> "voyez", "peux" -> "peut". The prompt also slows whisper.cpp: median inference 762 -> 1070 ms (tiny), 1626 -> 2078 ms (base), 5881 -> 7261 ms (small, 5 beams); runs differ by up to 20 % for other reasons (I-042), so the cost is real but its size is approximate. Accented prompts arrive intact (checked: the output spelling follows the prompt's accents).
- **Post-correction, engine-independent, applied to the clean run** (all 95 samples, 855 transcripts): STRICT (words of 6+ letters, same first letter, one edit up to 11 letters, two from 12): 44 changes in 43 transcripts, 0 words broken in all 9 configurations; Parakeet drug 4/6 -> 6/6 and WER 2.3 -> 2.1 %, Canary 4/6 -> 5/6, whisper.cpp small 3/6 -> 5/6 (3.9 -> 3.5 % and 4.5 -> 3.9 %), base greedy 2/6 -> 4/6 (11.4 -> 10.4 %). MEDIUM (5+ letters, one edit up to 9 letters, two from 10): 55 changes in 54 transcripts, 0 broken, a little more fixed (Canary 4/6 -> 6/6, 5.9 -> 5.0 %). LOOSE (4+ letters, three edits, any first letter): 365 changes in 290 transcripts and 26 to 38 words broken per configuration, WER Parakeet 2.3 -> 5.9 %, small 3.9 -> 7.2 %. The loose damage is exactly the predicted one: "matin" and "mais" turned into "main" (63 times), "après" into "API REST" (27 times: the window swallowed the next word), "reste" into "restore" (18) and "la convention" lost its article (18). Details in each folder's `post-corrections.jsonl`.
- **Combinations** (all 95 samples): prompt + strict correction on whisper.cpp small 5 beams: 3.9 -> 3.1 %, drug 3/6 -> 5/6, 5 words broken (all from the prompt, none from the correction); base greedy: 11.4 -> 9.1 %, drug 2/6 -> 6/6. Tiny gets no useful help from the prompt (54 words broken). Parakeet hotwords 1.5 + strict: drug 6/6, WER 2.5 %, 3 words broken.
- **Cost of the correction step**: it is a string pass on the text; its duration was not measured separately (not expected to matter next to inference).

**Failed or surprises**
- My first version of the correction swallowed a neighbouring word: "contreindiqué en" became "contre-indiqué" because a two-word term was compared to a one-word hypothesis plus the next word. The study caught it (1 broken word "en"); fixed by trying windows of one word fewer and one more and keeping the smallest distance (unit test added). A plural guard ("backups" must not become "backup") was added at the same time.
- I first believed "prednisolone" was one edit from "prednisone"; it is two. The strict preset leaves it alone, the medium preset replaces it silently (unit test `known_hazard_...` documents this look-alike pair; listing both drugs in the vocabulary is the only protection).
- The library calls were less hostile than feared: the NeMo hotword path did not terminate the process, it logs and skips hotwords it cannot encode.
- The comparison counts a word as "broken" when the alignment of a nearby error moves; examples are listed so a human can check. Some "broken" words in the prompt runs are real (a different word chosen), some are alignment side effects of a changed neighbour.

**Not verified**
- Anything beyond one speaker, one microphone and these 95 sentences; a vocabulary built from the test sentences is an upper bound. No test with a vocabulary that does NOT contain the answer (the 16 added drug names are never spoken in the dataset), no larger vocabulary (hundreds of terms), no real-world free text to measure false corrections of the dictionary step; the 0 broken words of the strict preset come from 95 short sentences.
- Statistical power: 6 drug occurrences. No interval was computed.
- Whether a proper SentencePiece vocabulary changes the hotword result; whether a hotword score between 1.5 and 3.0, or per-word scores, or a hotword form without the leading word marker (to keep the elision) does better; whisper.cpp `--carry-initial-prompt` and a prompt written as a sentence instead of a list; whisper.cpp with a larger model; Canary (no biasing mechanism was found for it).
- Timing cost of hotwords: median 551 ms (clean run, greedy) against 727 ms (beam-search control) and 710 ms (hotwords 1.5); the beam and the between-run offset (I-042) are not separated.

**Next**: private accent clips (backlog item 6, prompt `docs/prompts/04-private-accent-clips.md`). Owner decision needed: which technique, if any, to keep for the product (D-036 gives a recommendation, not a decision).

**Identity check (end of entry)**: the editor name that `bench preflight` stores in `config.json` was replaced by `code-editor.exe` in the nine new result folders. No private content was written (the study uses the owner's public-dataset recordings only; private clips were not used).

---

## 2026-10-08 — T5 (part 1) — Import tool for private clips; the evaluation itself waits for the owner's clips

**Done**
- Read-only checks: clean working tree on `milestone/t4-drug-name-handling` (HEAD `8b40ca4`, the T4 work is committed), no `bench.exe` or `whisper-cli.exe` running, `benchmark/samples-private/` empty, no private result folder yet. Read how private samples work (`dataset.rs`: `samples-private/`, the `private` flag set by the loader, `third-party-private` forced into the private folder; `bench run --include-private` writes to `results/private/`; `.gitignore` covers `benchmark/audio/`, `benchmark/samples-private/`, `benchmark/results/private/`).
- New library function `dataset::import_sample` (with `ImportRequest`, `private_voice_license`, `clean_reference`) and new subcommand `bench import`. It takes an existing PCM WAV and the owner's typed reference file, converts the audio to 16 kHz mono 16-bit (stereo averaged, other rates resampled by the existing linear-interpolation function of `chunking.rs`), writes `benchmark/audio/<id>.wav` and the metadata JSON (`samples-private/` for `--private`), then runs the normal dataset validation on that sample and rolls everything back if a check fails. No code change is needed per clip afterwards.
  - Privacy by construction: the reference file is read and never printed (the command prints only the id, duration, word count, category and destination; error messages never contain the reference, unit-tested). `--private` requires `--consent yes|unknown`; `--consent no` refuses the import. Public or licensed material needs `--source-kind`, `--license` and, for `public-domain`/`licensed`, a `--url`. An existing id is refused unless `--replace` is given. The id must already be in lower-case `[a-z0-9_-]` form.
  - Default category of an imported clip is `<language>-accent-private` (or `-accent` for public material), so `bench run --include-private --category fr-accent-private` selects exactly the imported clips and no other sample.
- Usage (documented in `benchmark/README.md`, `README.md` and the `bench` header): `bench import --wav <file> --id <id> --reference-file <txt> --speaker <id> --accent <text> [--gender <text>] --private --consent yes`.
- `cargo test --lib`: 123 passed (118 before; 5 new: conversion to 16 kHz mono and private placement, duplicate refusal and `--replace`, bad input leaves nothing behind and never echoes the reference, public material with a source, consent statement).

**Verified (observed here)**
- End to end with the release build on a synthetic tone (22.05 kHz mono, 30 s, written in a scratch folder through `SPEECHLAB_BENCH_DIR`, NOT the real dataset): the import produced a 30.0 s file and a JSON in `samples-private/` with `source.kind` = `third-party-private`, `committable` = false and the consent statement in `source.license`; `bench check` then reported 1 sample (1 private), 0 validation issues; a second import of the same id and an import with `--consent no` were both refused with a clear message; the 30 s clip printed the hint to use `--chunking vad`.
- The real dataset was not modified: `git status` shows only source and documentation files; `benchmark/samples-private/` is still empty.

**Failed or surprises**
- `bench check` warns "unusual reading speed: 0.1 words/s" on the synthetic tone with a 4-word reference: expected (a tone is not speech), and a useful sanity check that the warning logic still works on imported samples.
- The first attempt to append these documents with one long inline shell script was rejected by the agent shell (apostrophes, see the trap in HANDOFF section 5); nothing was written by that attempt, the text was then written through files.
- A trap found while reading the code, not fixed (I-047): `summary.md` of any run contains a "suspect samples" section that prints the reference and the best output of the hardest samples. For a private run it is written to `benchmark/results/private/<run>/summary.md` (git-ignored, so safe on disk), but it must never be copied into a document. `runs.jsonl` also holds every reference and transcript.

**Not verified**
- The evaluation itself (backlog item 6): no clip was imported, no private run was made, no accent figure exists. It needs from the owner, per clip: the file path, a general description of the speaker (accent, language, gender only if the owner states it), whether the speaker consented, and the verbatim reference typed by the owner. Nothing was invented, listened to or transcribed.
- Import of non-WAV files (m4a, mp3, Ogg): refused with the existing clear error; the owner converts them first (Audacity or VLC). Resampling quality of non-16 kHz clips (linear interpolation, never compared with a better resampler; I-046).

**Next**: the owner provides the clips and references, then a new session follows `docs/prompts/04-private-accent-clips.md` from its step 2 (the import tool of its step 3 already exists). M6 (text-to-speech laboratory, prompt `docs/prompts/05-tts-laboratory.md`) does not depend on the clips and can run first.

**Identity check (end of entry)**: no AI or vendor name was written; no private content exists in the repository (no clip was imported; the only test audio is a synthetic tone in a scratch folder outside the repository).

---

## 2026-10-08 — M6 — Text-to-speech laboratory (sherpa-onnx; Piper and Kokoro voices; D-038)

Context: the owner could not find the accent clips (T5 stays open, see the previous entry), so M6 was started; it does not depend on them.

**Done**
- Read-only checks first: clean tree on `milestone/t5-private-clip-import` (the T5 commit `0991905` made by the owner), no benchmark running. Read the TTS contract (`TextToSpeechProvider`, declared in M1, never implemented), the model manager, the registry, the commands and the UI patterns.
- Candidate list built from the official release metadata (asset names, sizes and SHA-256 digests of the `tts-models` release; metadata only, no model downloaded) and from the voices' own model cards (VERIFIED, see below). **The owner approved the download** of: Piper `fr_FR-siwis-medium` (67 MB), Piper `fr_FR-gilles-low` (67 MB), Piper `en_US-libritts_r-medium` (82 MB) and a Kokoro multi-language package. All four archives were installed through the project's downloader and the SHA-256 matched the published digest each time.
- **Mistake made and corrected**: I first proposed `kokoro-int8-multi-lang-v1_1` (the owner approved "Kokoro int8 v1.1"). After installing it, its README said "kokoro v1.1-zh" and the model's card confirms it supports English and Chinese only (2 languages, 103 voices): **no French**. I had marked its French support "to verify" in the feasibility report and did not verify it before proposing the download. I replaced it by `kokoro-int8-multi-lang-v1_0` (132 MB; 54 voices, exactly one French voice `ff_siwis`, documented female, trained on SIWIS, CC-BY 4.0), deleted the v1.1 folder, and told the owner. Same model family and a smaller download than approved, but it is a different version from the one approved: the owner should know.
- Code:
  - `models.rs`: role `tts` (voices are listed by `tts_models()` and never by `list()`, so the STT pickers and the benchmark do not see them), families `piper-vits` and `kokoro`, new manifest file fields `voices`, `lexicon`, `dataDir` (the phonemizer data folder must exist). Four manifest entries (checksums pinned, licences stated, espeak-ng noted).
  - `tts.rs`: `SherpaTtsProvider` implements `TextToSpeechProvider`. Voice ids are `<package>:<speaker>`. Voice list: Piper packages read their `*.onnx.json` (speaker count, sample rate; multi-speaker packages list the first 12 speakers, any number still works); Kokoro lists its French and English voices with the gender documented by its voice table. Synthesis returns a WAV file plus generation time, audio duration, RTF, load time, cold flag, sample rate, speed, peak memory. Real cancellation through the library callback. Text limit 5000 characters, speed 0.5 to 2.0, language and speaker validated before any model is touched. Generated files can only be read back from the TTS output folder (`read_generated`, unit-tested against path tricks).
  - `registry.rs` and `commands.rs`: TTS providers next to STT ones; commands `list_tts_providers`, `list_tts_models`, `list_tts_voices`, `synthesize`, `cancel_synthesis`, `read_tts_audio`, `clear_tts_audio`.
  - UI `TtsPanel.tsx`: voice package table with install buttons, language, voice (with gender and licence), speed slider, text, Generate/Cancel, then a SEPARATE player (never auto-play) with Stop, Save WAV, "Delete generated files" and the timing line.
  - `examples/tts.rs` (`voices`, `say`, `measure`), `scripts/tts_summary.py`, `scripts/tts_roundtrip.py`, sentence files in `benchmark/tts/`.
- Tests: `cargo test --lib` 132 passed (123 at the start of M6; 125 after the model-manager changes; 130 with the provider's unit tests and the registry test; 132 with the real-voice test and the output-folder guard); one test runs the real Piper voice (audio produced, speed ordering, cancel before start, cancel during a long text) and is skipped if the voice is not installed. `pnpm typecheck`, `pnpm test` (5) and `pnpm build` pass.

**Verified (observed here; CPU only, Intel Core 7 150U, release build)**
- **Licences read from the cards**: Piper siwis CC-BY 4.0; gilles CC0; upmc CC-BY-SA 4.0 (not downloaded); libritts_r CC-BY 4.0; Piper `en_US-ryan` CC-BY-NC-SA 4.0 (excluded); `en_US-lessac` points to the Blizzard 2013 licence page (not read, excluded for now); `en_US-amy` and `en_GB-alan` say only "See URL" (not read); `fr_FR-miro` has no card (404); Kokoro-82M weights Apache-2.0, `ff_siwis` listed as trained on SIWIS (CC-BY 4.0). Known exclusions stay: `fr_FR-tom` (AGPL), `vits-mms-fra` (CC-BY-NC).
- **Voices offered by the app** (4 packages installed): French: Piper siwis (1 voice), Piper gilles low (1 voice, 16 kHz output), Kokoro `ff_siwis` (female). English: Piper libritts_r (904 speakers, 12 listed, gender undocumented), Kokoro 28 voices (American and British, female and male per the voice table). Gender is "unknown" wherever the cards do not state it.
- **Intelligibility smoke test**: all generated audio was non-silent (peak 0.37 to 0.63) and Parakeet recognised the whole test sentence from each of the three voices first tried.
- **Speed control works on both families, differently**. Audio duration for the same French sentence (one run per speed, Piper siwis / Kokoro `ff_siwis`): speed 0.5: 6578 / 8664 ms; 0.8: 4674 / 5060; 1.0: 4040 / 4079; 1.25: 3448 / 3415; 1.5: 3111 / 2566; 2.0: 2600 / 2053 ms. Kokoro follows the setting almost linearly (2.0 gives 0.50 of the duration), Piper does not (2.0 gives 0.64, 0.5 gives 1.63): the setting is not a true factor for Piper. Pitch and quality at the extremes were not judged (listening).
- **Real cancellation**: cancelling a long text (40 sentences) stops generation in under half of the full time (test passes); in the app, Cancel returned "operation cancelled" within 2.5 s on a long Kokoro text. The first chunk of a request is not interruptible.
- **App test** (real app, DevTools protocol, port 1430 only): the panel lists the 4 packages as installed, the voices per language (French: 3, English: 40), switching the language swaps an untouched sample text, Generate produces a player that is PAUSED (no auto-play) with a duration, Cancel works. Not tested in the UI: pressing play (needs a human click), the Save WAV link (Blob download), the install button for a not-installed package (all four were already installed).
- **Speed and memory, 3 repetitions, 4 sentences per language** (run `benchmark/results/20261008-085403-tts-m6/`, preflight before: 17.4 % background CPU on mains power; a preflight taken right after the run read 33.2 %, above the 30 % limit, so the figures carry extra uncertainty, I-042; warm figures exclude the first generation after loading; sentences of 30 to 177 characters from the public dataset scripts; `summary.md` there has every row):
  - Piper (three voices): RTF 0.034 to 0.079 (median generation 66 to 784 ms for 1.5 to 11 s of audio), i.e. about 13 to 29 times faster than real time. Spread between repetitions 4 to 89 % because the times are tiny (66 to 400 ms).
  - Kokoro int8: RTF 1.55 to 2.06, **slower than real time** (3.4 s to generate 1.6 s of French, 18.3 s for a 10.4 s paragraph); spread 3 to 20 %. In the earlier smoke test (different moment, no preflight) the same voice read RTF 1.1 to 1.2: a 40 to 70 % between-run difference, the same effect as I-042.
  - Cold load: Piper 0.77 to 0.93 s, Kokoro 1.6 to 1.9 s. Peak resident memory of the process: Piper 210 to 257 MB, Kokoro 433 to 456 MB (sampled, a lower bound).
- **Machine intelligibility check** (`roundtrip-parakeet.txt` in the same folder; 18 files at speed 1.0, 3 sentences per language per voice, Parakeet as recogniser): the heard text of every file is a recognisable version of the sentence; numbers and dates came out right in all French files and mostly in English. Drug names recovered exactly (5 per voice and language: amoxicillin, ibuprofen, paracetamol, prednisolone, omeprazole): French Kokoro `ff_siwis` 4 of 5, Piper siwis 2 of 5, Piper gilles 2 of 5; English Kokoro `af_heart` 3 of 5, `am_adam` 3 of 5, Piper libritts_r 1 of 5. This is NOT a naturalness measure: the recogniser misspells drug names for human voices too (Parakeet found 4 of 6 in the human dataset), and one sentence per voice is anecdotal. The WER printed by the script is inflated by number formats ("500" against "cinq cents") and must not be quoted.
- **The same library carries the GPL phonemizer**: the TTS path needs the full sherpa-onnx static libraries (the "no-tts" ones used for the D-012 experiment cannot synthesise). Any build that ships TTS therefore contains espeak-ng (GPL-3.0). Nothing was changed for STT-only builds.

**Failed or surprises**
- The Kokoro v1.1-zh mistake above (wasted a 147 MB download).
- Kokoro French logs `Skip unknown phonemes. Unicode codepoint: U+002D` (a hyphen) from the library for the sentence tried first; its cause (a phoneme output, not a character of the text, which had no hyphen) was not isolated and its audible effect not judged (I-050).
- The first shell script used to append these documents was rejected by the agent shell (apostrophes); the text was written through files.
- Kokoro is far slower than its reputation suggests on this CPU with int8 weights and 4 threads. Not tried: fp32 Kokoro (334 MB), more threads, a smaller Kokoro, Kitten or Matcha models. Piper is the only family that is interactive here.
- The default 22.05 kHz Piper files and the 24 kHz Kokoro files need no resampling for playback; Piper gilles-low outputs 16 kHz (telephone-like by design, "low").

**Not verified**
- **Naturalness, accent, pronunciation of drug names and numbers by a human ear**: not judged by me. 24 listening files were generated in `benchmark/tts-samples/` (git-ignored; 6 voices, 3 sentences per language, plus the first sentence at speed 1.5). The owner's notes are PENDING and must be recorded verbatim as opinion.
- macOS and Windows packaging behaviour of the TTS libraries and of the `espeak-ng-data` folder (M7). A distributable layout for the phonemizer data was not designed.
- Pause/resume in a real click session; export file saved from the WebView; install button for a new package.
- Texts over 5000 characters, SSML, numbers read by the voice (digits versus words) beyond the sentences tried, French accents of the Kokoro voice beyond one voice, licences of the Kokoro training data for the English voices (the voice table lists mixed sources; not read in detail), the CC-BY-SA upmc voices, fp32 Kokoro, other threads settings, GPU.
- Statistical power: 3 repetitions, 4 sentences per voice, one machine.

**Next**: the owner listens to `benchmark/tts-samples/` and gives notes (to record as opinion); T5 accent clips when the owner finds them (`docs/prompts/04-private-accent-clips.md`, step 2); then M7 packaging (`docs/prompts/06-packaging-validation.md`).

**Identity check (end of entry)**: the editor name that the quiet-machine check records was replaced by `code-editor.exe` in the new `config.json`. No private content was written (all test texts come from the public dataset scripts or are written for this study).

---

## 2026-10-08 — M6b — Licence-first orientation of the TTS benchmark (D-039); reading-highlight analysis (D-040, proposed)

Owner's instruction (after M6 was committed): the voices must be judged first by how free they are for commercial use, and the benchmark oriented that way; also, is "highlight the text being read" planned?

**Done**
- **Licence review of every voice examined** (new `docs/TTS_LICENSES.md`, rule: read the card INCLUDING "Finetuned from" and follow the parent's licence; rate each voice `clear` / `attribution` / `review` / `excluded`; the phonemizer is rated separately). Sources: voice cards, the Blizzard 2013 licence page, Kokoro and CSS10 repositories, Coqui and NeonGecko model cards, the `config.json` of the Coqui package.
- **The rating is now part of the product, not only of a document**: manifest fields `licenseTier`, `licenseNotes`, `phonemizer`; carried in `ModelInfo` and `VoiceInfo`; shown in the TTS tab (colour-coded "Commercial use" column with the reason on hover, `[rating]` in the voice list, a warning under a `review` or `excluded` voice); a unit test refuses a voice without a rating, a reason or a stated phonemizer.
- **Two more voices installed (owner-approved download)** to have the cleanest candidates: English **Piper ljspeech medium** (public domain, trained from scratch: `clear`) and French **Coqui VITS css10** (BSD-3-Clause model, LibriVox public-domain recordings: `attribution`). The Coqui release asset publishes no digest: its SHA-256 was computed at the first download and pinned (trust on first use, D-015), then re-checked by the installer. New model family `coqui-vits` (character input, no phonemizer data folder).
- **Measured again with all 8 voices in ONE run** (`benchmark/results/20261008-094413-tts-m6-licence-focus/`, same method as M6, comparisons are valid inside this run only, I-042).
- **Listening session tool**: `scripts/tts_listening.py` writes `benchmark/tts-samples/index.html` (git-ignored): 8 voices under anonymous codes in a fixed shuffled order, 4 players each (nothing auto-plays), a form per voice, "Copy my notes" for the chat. It replaces the loose list of 24 files.

**Verified (observed here)**
- **My earlier classification was wrong, and is corrected**: D-038 and the first M6 entry presented Piper `siwis` as CC-BY and `gilles` as CC0, i.e. as the low-constraint choices. That read only the "Dataset" line of the cards. The cards also say `siwis` was fine-tuned from the English **Lessac** voice (data under a Blizzard 2013 **research** licence agreement) and `gilles` from the English **Ryan** voice (data **CC BY-NC-SA 4.0**). Ratings now: `siwis` = review, `gilles` = excluded (kept installed for comparison). Same corpus note: Piper `gilles` and the Coqui css10 voice are both trained on the CSS10/Kaggle French single-speaker corpus (LibriVox recordings, speaker "Gilles G. Le Blanc"); only the starting checkpoint differs.
- **Kokoro**: weights Apache-2.0, but its model card lists as training data "audio licensed under Apache, MIT, etc" AND "synthetic audio generated by closed TTS models from large providers" (terms of those providers not read), plus SIWIS (CC BY 4.0) and Koniwa (CC BY 3.0): rating `review`. Its English voice `am_adam` is graded F+ by its own voice table (weak data); `ff_siwis` is graded B-.
- **Current ratings of the six installed packages**: ljspeech `clear`; Coqui css10 and libritts_r `attribution`; siwis and Kokoro `review`; gilles `excluded`. Not installed but identified: Piper `en_US-kristin` (public domain, from scratch, in the release, 64 MB); Piper `fr_FR-mls` (CC BY 4.0, from scratch, NOT in the sherpa-onnx release: needs a conversion).
- **The Coqui voice needs no phonemizer**: its package has no `espeak-ng-data` folder, its `config.json` says `use_phonemes: false`, and it loads and speaks with no data folder configured. The price (below): no text normalisation. The standard sherpa-onnx libraries still contain the espeak-ng code (open question, I-054).
- **Coqui drops digits**: on the sentence "Rendez-vous le jeudi 12 mars à 14 h 30 ; ..." the recogniser heard "Rendez-vous le jeudi, Mar avait apporté votre carte vitale ...": "12" and "14 h 30" were not spoken (the audio is also much shorter than for the other French voices: 2.7 s against 3.8 to 4.3 s). Every other voice spoke the date. A character-based model needs the application to spell out numbers, dates and units before synthesis (I-055).
- **Speed and memory, 8 voices, 3 repetitions, 4 sentences** (preflight 23.0 % before and after; **a `speechlab.exe` window that I did not start was open during the run**, so treat figures as approximate): Piper and Coqui RTF 0.035 to 0.064 (Coqui 0.046 to 0.051; 16 to 28 times faster than real time), cold load 0.78 to 1.29 s, process memory 212 to 258 MB; Kokoro int8 RTF **1.10 to 1.33**, cold load 1.67 to 1.80 s, 432 to 456 MB. **The first M6 run gave Kokoro 1.55 to 2.06 on the same machine**: a 40 to 55 % difference between runs (I-042 again); the safe statement is "Kokoro int8 is slower than real time here, by a factor 1.1 to 2.1 depending on the run". Piper ljspeech 0.052 to 0.064, libritts_r 0.056 to 0.061, siwis 0.045 to 0.050, gilles 0.035 to 0.037.
- **Synthesis is not deterministic**: regenerating the same text gave different audio and, through the recogniser, different words (for example Piper siwis heard "amoxycilline" in the first set and "oxycyline" in the second; Kokoro `ff_siwis` 4 of 5 drug names in the first set, 5 of 5 in the second). The machine-intelligibility counts are therefore anecdotal by about plus or minus one per voice. Second set, drug names recovered exactly out of 5: French Kokoro 5, Piper siwis 3, Coqui 2, Piper gilles 1; English Kokoro `af_heart` 3 and `am_adam` 3, Piper ljspeech 2, libritts_r 1. Numbers and dates were right everywhere except the Coqui French date. The `clear`/`attribution` voices are NOT the best at drug names in this check; no voice is good on all five; all are rated by a recogniser that itself fails on drug names.
- Tests: `cargo test --lib` 132 passed, `pnpm typecheck`, `pnpm test`, `pnpm build` pass. The manifest test now enforces ratings.

**Failed or surprises**
- My earlier "free licence" statements were incomplete (above). Lesson recorded as a rule in `docs/TTS_LICENSES.md`: follow the "Finetuned from" chain.
- The Coqui page of the sherpa-onnx documentation could not be fetched (a 151-character answer); the package was inspected after the download instead.
- A `speechlab.exe` process that was not mine appeared during the run (probably the owner's window); it was not touched. Its effect on the figures is unknown.

**Not verified**
- Whether weights fine-tuned from a research-licence or NC checkpoint inherit the restriction (legal question, I-052); the clauses of the Blizzard licence on derived models (only the first part of the page was read); the terms of the closed TTS providers whose output trained Kokoro (I-053); whether shipping the espeak-ng code that a character-based voice never calls is acceptable (I-054); the BSD-3 notice and CC-BY credits wording.
- Naturalness of all voices (owner's listening: PENDING), the audible effect of the Coqui voice's missing normalisation beyond digits (abbreviations, units, acronyms), the Coqui voice's lineage (card silent).
- Whether a text-normalisation step for French and English numbers, dates, times and units gives the Coqui voice acceptable behaviour (not built).

**Reading-position highlight (analysis only, D-040 proposed, nothing built)**: not in `Plan.md` (searched) and not in the current UI. The synthesis API returns samples and a sample rate only, no word timings (checked in the crate). Details and options in D-040; the owner chooses the granularity.

**Next**: the owner listens with `benchmark/tts-samples/index.html` and pastes the notes; the owner chooses the highlight granularity; decide on `kristin` (English) and a French text normaliser; M7 packaging (`docs/prompts/06-packaging-validation.md`, which now carries the licence rules).

**Identity check (end of entry)**: the editor name recorded by the quiet-machine check was replaced by `code-editor.exe` in the new `config.json`. No private content was written.

---

## 2026-10-08 — M6c — Owner's listening notes and decisions (highlight, normaliser, beta licence stance); `kristin` installed

**Done**
- **The owner's blind listening notes are recorded verbatim** in `docs/TTS_LISTENING_NOTES.md` (opinion, one listener, 8 voices A to H). Owner's choice: Piper siwis (B), Piper gilles (D), Piper ljspeech (G), Piper libritts_r (H). Analysis kept apart from the opinion (see the file): English has no quality/licence conflict, French does; Kokoro has an audible hiss; speed 1.5 is acceptable everywhere.
- **Owner's UI test result**: "Tout fonctionne parfaitement" for the nine checks proposed (voice table with colours, rating warning, no auto-play, Coqui digits, speed, cancel, player, Save WAV, language switch). This is the owner's report; I did not re-observe it (the owner's dev session was open, so I did not drive it).
- **Decisions of the owner (details in D-040 accepted and D-041)**: (1) highlight the sentence being read, grey background; (2) automatic scrolling for long texts; (3) click on a sentence to jump is NOT needed for a first release, possible later for an add-on or an editorial use case with a lot of text; (4) if Coqui is the only free French voice, build the digits-to-words normaliser; (5) install `kristin` and prefer public-domain material at this stage; (6) a beta version is acceptable only if it needs no paid licence, otherwise abstain; a budget for legal advice or licences may be considered later depending on users and revenue; (7) the accent clips were not found.
- **`kristin` installed** (Piper en_US kristin medium, public domain LibriVox recordings, trained from scratch per its card, rated `clear`; official digest verified; 67 MB). Smoke test: RTF 0.056, 258 MB. Not yet measured in a controlled run, not yet listened to.
- Tests after the manifest change: `cargo test --lib` 132 passed.

**Verified**
- Everything listed under "Done" about files and the installation (digest match, voice listed, one synthesis run).

**Failed or surprises**
- The voices the owner likes best in French (siwis, gilles) are exactly the ones the licence review could not clear (I-052). The free French voice is the weakest one (clarity 3, accent 2). This is the main open problem of M6 and drives the next work.
- Kokoro's hiss (I-057) was heard on all three Kokoro voices; it was not detected by any of my measurements, only by ear.
- T5 (accent clips) cannot proceed: the owner has no clips. `bench import` remains ready and tested.

**Not verified**
- Quality and speed of `kristin` beyond one smoke test; the cause of the Kokoro hiss; whether a text normaliser makes the Coqui voice acceptable (not built); whether the Piper French `mls` voice (CC BY 4.0, trained from scratch, not in the sherpa-onnx release) sounds better than Coqui after conversion; whether the `review` voices become usable (needs a legal reading, postponed by the owner).

**Next**: prompt `docs/prompts/06b-tts-readalong-and-normaliser.md` (sentence highlight with auto-scroll, text normaliser for the Coqui voice, `kristin` measurement and listening, search for a better free French voice); then M7 packaging (`docs/prompts/06-packaging-validation.md`).

**Identity check (end of entry)**: no AI or vendor name added; the owner's opinions are quoted from their own text.

---

## 2026-10-08 — M6d — Read-along highlight, sentence splitter, text normaliser, `kristin` measured, `mls` voice converted

**Done**
- **Sentence splitter** (`src-tauri/src/speech/sentences.rs`, 11 unit tests): French and English in one set of rules (abbreviations M., Mme, Dr, Pr, St, etc., env., vs, e.g., month abbreviations; decimals, times, dotted acronyms; ellipses; guillemets with the French thin space; list numbers; line breaks and blank lines = paragraph; punctuation-only pieces glued to the previous sentence). Returns each sentence's span in the ORIGINAL text in UTF-16 code units (JavaScript string indices), checked against `String.encode_utf16` with accents and an emoji.
- **Text normaliser** (`src-tauri/src/speech/normalise.rs`, 15 unit tests, French and English, no new dependency): cardinals (70 / 80 / 90 as "soixante-dix", "quatre-vingts" with the plural only when nothing follows, "quatre-vingt-un", "et un" / "et onze"), ordinals (1er, 2e, 21e, 12th), decimals, thousands separators, negatives, ranges, percentages, currency (euro, dollar, pound, CHF), times (14 h 30, 9h00, 14:30, "10.30" only after a time word, 9 am), dates (12 mars 2026, 12/03/2026, 2026-03-12, March 12th, 2026), fractions, "3 x par jour", units with the plural rule of each language (mg, ml, kg, °C, mmHg, bpm, UI, mg/kg, mg/jour, ...), abbreviations, phone numbers (groups of two digits as numbers, other groups digit by digit, commas between groups) and long identifiers (digit by digit). Applied per sentence, only to the text sent to the voice. Switched on by the manifest flag `normalizeText` (true for Coqui css10, false for the phonemizer voices; a unit test ties the flag to the phonemizer field); the request can override it (`normalise: true|false`).
- **Per-sentence synthesis with timings** (`tts.rs`): `plan_sentences` (one function for the synthesis, the preview and the highlight) -> one library call per sentence -> joined into ONE WAV (16-bit mono written with `hound`) with 250 ms after a sentence and 600 ms after a paragraph, both divided by the speed factor. `SynthesizeResult` gains `segments` (original text span, start ms, end ms) and `normalised`. Cancellation works between sentences and inside one (callback); the existing cancel test still passes on a 40-sentence text. `synthesize_whole_text` (the old single call) exists for measurement only.
- **Interface** (`ReadAlong.tsx`, `readAlong.ts`, `TtsPanel.tsx`, `styles.css`): after Generate the text becomes a read-only view of sentence spans; the sentence being read has a grey background (`aria-current`), driven by the player's clock on every animation frame; pause keeps it, Stop clears it, seeking with the player's bar moves it, nothing is shaded before the first play or after the end; automatic smooth scrolling keeps the sentence in view (reduced-motion respected), a manual scroll (wheel, touch, scroll bar) suspends it for 4 s, a "Follow reading" checkbox turns it off; "Edit text" returns to the text box and the highlight is invalid until the next Generate. A checkbox shows and overrides the rewriting of numbers (default from the voice) and "Show the text sent to the voice" lists each sentence as the voice receives it (new command `preview_spoken_text`). Segment indices and their times are on the spans (`data-segment`, `data-start-ms`, `data-end-ms`), so click-to-jump can be added later.
- **Tests**: `cargo test --lib` 163 passed (132 before; one of the new ones uses the real Piper voice), `pnpm typecheck`, `pnpm test` 16 passed (12 new for the pure functions: time to sentence index, text pieces, scroll target), `pnpm build` pass.
- **UI test over the DevTools protocol** (`scripts/ui_test_readalong.mjs` and `scripts/cdp.mjs`, port 1430 and 9222 only, audio muted, the files it created deleted): 33 checks passed, 0 failed (details below).
- **Overhead and `kristin`** (run `20261008-141002-tts-m6d-overhead`, preflight 10.5 % before and 21.8 % after, never forced, 5 repetitions, both modes interleaved in each repetition, cold runs excluded): see the table below.
- **Normaliser on the real Coqui voice** (rough recogniser round trip, `scripts/tts_normaliser_check.py`, Parakeet, 6 French sentences, 3 repetitions, two complete runs): key numbers heard 0/36 with the normaliser off, 28/36 then 30/36 with it on.
- **Research for a better free French voice** (read-only, no download except what the owner approved): see "Candidates" below and `docs/TTS_LICENSES.md`.
- **Piper `fr_FR-mls-medium` converted** with the owner's approval (isolated environment `vendor/py-convert`, only the `onnx` package, 99 MB; source files 76.7 MB from the Piper repository; `scripts/convert_piper_voice.py`). The procedure was first verified on two OFFICIAL converted packages (kristin, ljspeech): identical `tokens.txt` and identical metadata. The voice loads and speaks in sherpa-onnx (RTF 0.09 warm, 298 MB). A manifest entry (`packaging: "local"`, `attribution`) and a new packaging kind `Local` (install never downloads, never deletes the folder; unit-tested).
- **Listening session 2 prepared** (`python -I -X utf8 scripts/tts_listening.py --session 2`, page `benchmark/tts-samples/session2/index.html`): English kristin, ljspeech, libritts_r; French Coqui normaliser ON and OFF, and three `mls` speakers (24, 65, 117, the best by a recogniser proxy: 33 % word error rate each).

**Verified (with evidence)**
- Splitter, normaliser, per-sentence provider: the 163 unit tests (cargo output). On the real Coqui voice the spoken text for "Prendre 500 mg trois fois par jour. Rendez-vous le 12 mars à 9h00." was "Prendre cinq cents milligrammes trois fois par jour. Rendez-vous le douze mars à neuf heures." (`tts say --show-text 1`).
- Segments: with the real Piper voice, three sentences gave three segments whose text spans, cut with UTF-16 offsets, equal the sentences; the gaps equal 250 ms and 600 ms (to 2 ms); the last segment ends at the audio length; the WAV length read back equals the reported length (within 1 ms); at speed 2.0 the pause is shorter; a one-sentence request gives one segment `0..8` starting at 0 (unit test `real_voice_returns_ordered_segments_that_match_the_wav`).
- Overhead of sentence-by-sentence synthesis (median generation time, per-sentence minus whole text, same process, same voices; full table in `overhead.md`): 5-sentence texts +4 % (Coqui), +1 % (kristin), 0 % (libritts_r), -1 % (ljspeech); 10-sentence texts +2 %, +1 %, +5 %, -7 %; single sentences between -14 % and +16 % (noise of 100 to 250 ms runs). Conclusion: the extra cost is within the run-to-run spread (1 to 4 % within a run, I-042), at most a few percent. The audio is 7 to 9 % longer on 5 and 10-sentence texts because of the inserted pauses (e.g. 10 sentences, +2.0 to 2.6 s).
- Speed of the voices measured together (warm RTF, per-sentence mode, 5-sentence and 10-sentence paragraphs): Piper ljspeech 0.040 to 0.052, libritts_r 0.056 to 0.060, kristin 0.055 to 0.058, Coqui css10 0.058 to 0.060 (about 17 to 25 times faster than real time); peak memory median 226 MB (ljspeech), 258 MB (libritts_r), 197 MB (kristin), 226 MB (Coqui). `kristin` is as fast as the other Piper voices; its naturalness awaits the owner's ear.
- UI (33 checks, `ui-test-results.json`): no sentence shaded and no playback after Generate; six sentences in order, "Dr." did not split a sentence; seeking to the middle of each of the six sentences shades exactly that one (0 to 5 each); a sentence stays shaded during the pause after it and the next one is shaded from its first millisecond; during real (muted) playback the shading moved through sentences 0 and 1 in 4.5 s; pause keeps the same sentence; Stop clears it and rewinds to 0; seeking after Stop shades again; exactly one `aria-current`; contrast of the shaded text 12.7 (light) and 8.5 (dark), plain text 13.4 (dark), read from the computed colours (the shading differs from the box background by 1.48 and 1.97 as a ratio); "Edit text" brings back the text box and a changed text shows the "generate again" hint; 40 sentences: the box is scrollable (506 px), a sentence already on screen does not move it, sentences 20, 30 and 39 scrolled it to 273, 463 and 506 px with the shaded sentence fully visible, seeking back returned it to 0, a manual wheel event suspended the scrolling (0 px to 0 px) and it resumed after 4 s, "Follow reading" off kept the box still; French Coqui voice: the rewriting box is checked by default, the preview shows the rewritten sentences, unchecking shows the original, the displayed text keeps the digits and the result line says the text was rewritten; the phonemizer voice (siwis) has the box unchecked. Screenshots of the light and dark blocks were looked at.
- Coqui digits: the recogniser heard the key numbers in 0/36 cases with the normaliser off (digits dropped, e.g. "La facture numéro est payable sous jour") and in 28/36 and 30/36 with it on ("La facture numéro 2045 est payable sous 30 jours" three times out of three).
- Conversion procedure: `convert_piper_voice.py check` on kristin and ljspeech: tokens identical (157 lines) and the seven metadata keys identical. The converted `mls` file is the source plus 129 bytes of metadata.

**Failed or surprises**
- The first UI runs showed 2 failures that were bugs of the TEST, not of the app: the speech-to-text panel also has a (disabled) "Stop" button that my script clicked first (the TTS buttons now have ids), and sentence 10 of the long text is already visible without scrolling, which is the correct behaviour (the check was rewritten). A preview check failed once because the page kept a checkbox state from the previous run (the script now reloads the page first).
- The recogniser round trip is NOT clean even with the normaliser on: the date sentence is still mangled by the Coqui voice ("Rendez-vous le 12 mars à 9h00" heard as "J'ai voulu doucement à 9 heures", "Rendez-je vous les douze morts à 9 heures"): the voice's own articulation, not the rewriting (its spoken text was correct). The long phone number was heard as "06 1234 56 78". The Coqui voice is usable for numbers now, not good.
- `mls` is much weaker than Coqui by the same machine proxy: of 125 speakers, 74 had a word error rate of 90 % or more (several returned nothing), only 18 were at 50 % or better, the best three at 33 %; references: Coqui 6.7 %, Piper siwis 6.7 %, Piper gilles 20 % (one pair of sentences, one recognition each). The proxy says nothing about naturalness, so three speakers go to the owner's ear, but `mls` is not an obvious fix for the French voice.
- The owner's `pnpm tauri dev` recompiled and restarted its app each time I edited a Rust file (file watcher), until the owner closed it. Stopping my own `pnpm tauri dev` task did not stop its child processes (`speechlab.exe`, `cargo`, `pnpm`, the Vite `node`): I identified them by command line (all from this repository and started by me) and stopped them explicitly; the owner's other processes were not touched.
- First `bench preflight` of the session: 32 % CPU, not quiet (limit 30 %); two minutes later 10 to 12 %; measurements were run only on quiet checks.
- Shell traps again: a Python heredoc turned `\n` inside Rust string literals into real newlines (compile error "character constant must be escaped"); a long heredoc with apostrophes was rejected by the shell. Both were solved by writing script files. The manifest and some sources are CRLF in the working copy and LF in the index (autocrlf): edits normalise to LF, `git diff` stays clean.
- Python 3.13 has no wheel for the `onnx==1.17.0` / `onnxruntime==1.17.1` pins of the sherpa-onnx guide: the latest `onnx` (1.23.2) was used and `onnxruntime` was not installed (the script does not need it); safe because the output was compared with official packages.
- A web page summary said the `mls` voice was "Mozilla Common Voice"; the voice card itself says OpenSLR 94 (Multilingual LibriSpeech), as in `docs/TTS_LICENSES.md`.

**Candidates for a better free French voice (research, read 2026-10-08; nothing downloaded except `mls`)**
| Candidate | Licence read | Lineage / data | Runtime and phonemizer | Size and CPU | Verdict |
|---|---|---|---|---|---|
| Piper `fr_FR-mls-medium` | data CC BY 4.0 (OpenSLR 94, LibriVox), card: trained from scratch | clean | sherpa-onnx after a metadata conversion, espeak-ng (GPL) | 76.7 MB, RTF 0.09, 125 speakers of very unequal quality | installed locally; to be judged by ear |
| Chatterbox multilingual (Resemble AI) | MIT weights | "0.5M hours of cleaned data from freely available data on the internet": no data licence stated | PyTorch / ONNX / GGML (`chatterbox.cpp`, MIT), not sherpa-onnx; every output carries the Perth neural watermark; voice cloning from a reference clip | 0.5B, files about 2.1 GB (T3 1.1 GB + S3Gen 1.0 GB); the port reports RTF 4.32 for the multilingual model on an Apple M4, 4 threads (slower than real time) | NOT recommended: unstated data licence, far slower than real time on a laptop CPU (not measured here, NOT VERIFIED) |
| Chatterbox-TTS-French (`Thomcles`) | card: CC BY 4.0 (data), MIT (base) | trained on 1,400 h of the French part of the Emilia dataset; Emilia is CC BY-NC-4.0 (only its YODAS subset is CC BY 4.0) and the audio comes from videos and podcasts whose copyright stays with their owners; the card does not say which subset | as above | as above | `review`/likely `excluded`: the training data licence is unclear (NOT VERIFIED which subset) |
| Qwen3-TTS 0.6B / 1.7B | Apache-2.0 | "over 5 million hours of speech data" in 10 languages including French: no data licence stated | PyTorch (card shows CUDA and flash-attention); community GGUF and ONNX ports exist (llama.cpp style, `qwentts.cpp`, LunaVox), voice cloning from a reference clip | 0.9B parameters in BF16 for the 0.6B model; no CPU real-time factor found (RTF 0.35 on an RTX 5050 GPU only) | NOT recommended for now: unstated data licence, GPU-oriented, CPU speed unknown (NOT VERIFIED) |
| sherpa-onnx release | the release lists only `vits-coqui-fr-css10` and `vits-mms-fra` (non-commercial) for French beyond what is already installed (page read partially: the first 100,000 of 988,351 characters, 74 assets counted) | | | | no new French voice there |

**Not verified**
- The naturalness of `kristin`, of `mls`, and of the Coqui voice with the normaliser: the owner's listening session 2 is PENDING (steps given to the owner).
- The normaliser's quality on texts other than the examples (limits: acronyms such as IRM or ECG are not spelled, "un/une" only after a short list of feminine nouns, Roman numerals not converted, "10.30" is a time only after a time word, "numéro 2045" is read as a number, a list of numbers separated by single spaces such as "5 100" is read as one number); the English side has the same unit tests but no real character-based English voice to hear it.
- The read-along in the owner's own window (the UI test was automated, muted, in the development WebView); behaviour in a release build (M7) and on macOS.
- The dark theme: the page itself has no dark theme (its colours are fixed light); only the read-along block follows the system setting, so in dark mode it is a dark block on a light page (I-059).
- Timings on other machines; the overhead conclusion is for this CPU and these four voices.
- Whether shipping a converted `mls` voice needs anything beyond the credit (CC BY 4.0) and the espeak-ng point already open (I-051, I-054).

**Next**
- The owner does listening session 2 and pastes the notes (steps in `docs/TTS_LISTENING_NOTES.md`), then M7 packaging (`docs/prompts/06-packaging-validation.md`).

**Identity check (end of entry)**: the repository was searched for AI and vendor names (only the pre-existing mentions of the upstream Whisper weights' licence and the Opus codec remain; nothing added); the editor name in this run's `preflight-*.json` was replaced by `code-editor.exe`; no private content; `git status` lists no model, audio or `vendor/` file.

---

## 2026-10-08 — M6e — Owner's listening session 2 and interface tests; French licence problem re-examined

**Done**
- **The owner's session 2 notes are recorded verbatim** in `docs/TTS_LISTENING_NOTES.md` (scores, the owner's note on the interface tests with an English gloss, analysis kept apart).
- **French licence problem re-examined with sources** (read-only, nothing downloaded): the complete list of French voices of the sherpa-onnx release (`gh api`, 644 assets), the Blizzard 2013 licence, the Piper maintainer's answers (discussions 271 and 94), the `piper-checkpoints` repository (clean base model, siwis and mls checkpoints), the SIWIS dataset licence, the tjiho and miro cards. Details and the options in `docs/TTS_LICENSES.md` ("French voice: lineage problem and ways out"), D-043 (proposed), I-063.

**Verified (with evidence)**
- The owner's conclusion is exact: the French voices judged good (siwis, gilles) are exactly the ones with a restricted parent checkpoint (cards: "Finetuned from ... lessac", "Finetuned from ... Ryan"); the Coqui voice with the normaliser scores 3 out of 5 and the converted `mls` speakers 1 to 2.
- A clean public parent checkpoint exists (`rhasspy/piper-checkpoints/_base_model`, card: trained from scratch on LibriTTS-R, CC BY 4.0, repository MIT); the Piper maintainer wrote that he would re-train voices from it so that they can be used commercially. The SIWIS dataset is CC BY 4.0 with "usable for any purpose" in its README.
- The release's French voices are tom and tjiho (AGPL-3.0, the `LICENSE.txt` of tjiho read), upmc (CC BY-SA, from Lessac), miro (non-commercial, trained on synthetic data), siwis, gilles, Coqui css10, mms (non-commercial).

**Failed or surprises**
- **My earlier statement was wrong**: the M6d entry and `docs/TTS_LICENSES.md` said the sherpa-onnx release lists only `vits-coqui-fr-css10` and `vits-mms-fra` for French. A page summary had covered only the first 100,000 of 988,351 characters. The real list (644 assets) has more French voices, none usable, so the conclusion stands but my evidence was incomplete. Corrected in `docs/TTS_LICENSES.md`.
- The owner's usage differs from my default: the rewriting of numbers, dates, times and units is wanted on the phonemizer voices as well (D-043 proposes making it the default everywhere).

**Not verified**
- Whether weights fine-tuned from the Lessac or Ryan checkpoints inherit a restriction (legal question, untested according to the maintainer); the effort, cost and quality of re-training siwis and gilles from the clean base (no GPU on this machine, nothing tried); whether the gilles data is the CSS10 French corpus (only the Kaggle dataset title was checked); the licence terms of Piper's training code for a shipped product (it is not shipped, GPL-3.0, not read in full).

**Next**
- The owner chooses a way out of the French problem (D-043). Then either a training guide for a GPU session outside this machine, or M7 packaging (`docs/prompts/06-packaging-validation.md`, its step 0 no longer applies: the notes are recorded).

**Identity check (end of entry)**: no AI or vendor name added; third-party project names (Piper, Lessac Technologies, OpenVoiceOS) appear only as the voices' sources and licensors.

### 2026-10-08 — M6e addendum — Paid French voices (owner's question), rewriting default approved
- The owner asked whether paid French voices exist and at what price. Answer and table in `docs/TTS_LICENSES.md` ("Paid French voices"): cloud offers (Google, Amazon, Azure, OpenAI, ElevenLabs) cost $4 to $160 per million characters, ElevenLabs about $0.04 to $0.08 per 1,000 characters; only on-device SDKs (Acapela, ReadSpeaker, CereProc, Cerence) respect the offline rule and they publish no price. NOT VERIFIED: prices come from search results and third-party sites, not from the vendors' pages; nothing was listened to or tested.
- The owner approved the proposal of D-043 to turn the rewriting of numbers, dates, times and units ON by default for every voice, and asked for no coding for now: it stays to do (manifest `normalizeText` true everywhere, adjust the unit test in `models.rs`, keep the checkbox).

### 2026-10-08 — M6e addendum 2 — Owner's decision on the French voices (D-043)
- **Decision (owner)**: keep Piper siwis and gilles now, re-train them from the clean base checkpoint later, when the software is to be commercialised. Recorded in D-043 ("Owner's decision"), I-063 (update) and `docs/TTS_LICENSES.md`. D-041 is amended for these two voices until commercialisation; their ratings and the in-app warning are unchanged.
- Number rewriting ON by default for every voice: approved, coded in the next step (see addendum 3).
- Not recorded on purpose: the owner's comparison with another assistant's answer about the Piper licences (the owner asked to wait). The paid-voice table was recorded before that request.
- Not verified: whether a free beta given to others is non-commercial use (legal question); re-training time, cost and quality (nothing tried).

### 2026-10-08 — M6e addendum 3 — Number rewriting ON by default for every voice (D-043)

**Done**
- `normalizeText` is now `true` for all 8 voice packages of `models-manifest.json` (it was true only for Coqui css10). The manifest unit test in `models.rs` now requires it on for every voice instead of tying it to the phonemizer field. The checkbox stays and the request can still switch it off (`normalise: false`). The UI label no longer says that phonemizer voices read digits themselves ("on by default; the Coqui voice does not read digits at all"). The read-along UI test script now expects the box checked for the Piper siwis voice (it expected it unchecked).
- The real-voice unit test asserts that a Piper siwis request with no override reports `normalised = true` and that `normalise: Some(false)` gives `normalised = false`.

**Verified**: `cargo test --lib` 163 passed (including the two tests that use the real Piper siwis voice), `pnpm typecheck`, `pnpm test` 16 passed, `pnpm build`, `node --check` of the UI test script.

**Not verified**: the read-along UI test (`scripts/ui_test_readalong.mjs`) was NOT re-run after this change: the owner's own `pnpm tauri dev` session was open on port 1430 (not touched), so the default state of the checkbox on a phonemizer voice has not been observed in the real page since the change. How the rewriting sounds on siwis, gilles, libritts_r, kristin and ljspeech is the owner's own observation (their interface tests used it); the English rules were never listened to on a character-based English voice. Steps to verify: close the app, start it with the remote-debugging switch (`docs/HANDOFF.md` section 5), run `node scripts/ui_test_readalong.mjs`; expect 33 of 33.

**Next**: M7 packaging (`docs/prompts/06-packaging-validation.md`).

---

## 2026-10-08 — M7 — Windows packaging validation (D-044, D-045); macOS checklist (not verifiable here)

**Done**
- Read-only checks (step 1): no `bench.exe`, `whisper-cli.exe` or `tts.exe` running; the owner's own `pnpm tauri dev` window of this project was open (`speechlab.exe` from `target\debug` and four `cargo` processes of the owner's other application, untouched) and was closed by the owner before the build; free disk 556 GB before and 553 GB after; `tauri-cli` 2.12.1, `rustc` 1.98.1; the NSIS and WiX 3.14 tools were already cached in `%LOCALAPPDATA%\tauri` (nothing installed). Antivirus: Avast is the active product, Defender's real-time protection is off.
- Design proposed in the chat and chosen by the owner: NSIS `.exe` only, `whisper-cli.exe` as an `externalBin` sidecar with its four libraries as resources, owner's go for the first build (D-044).
- Configuration: `src-tauri/tauri.conf.json` (`bundle.active`, NSIS per-user, sidecar, four libraries as resources, icons; later `csp` and `devCsp`, D-045), `scripts/stage-whisper-sidecar.ps1` (copies the git-ignored whisper.cpp build into the git-ignored `src-tauri/binaries/`), `.gitignore`.
- Builds (`pnpm tauri build`, three times: first, then two CSP iterations): **155 s, 128 s, 115 s**. Compile of `tauri` and `speechlab` took 2 min 18 s in the first build; **the dependencies were already compiled in `target/release` by earlier release builds, so this is NOT a from-scratch build time**. Installer **8,858,030 bytes (8.45 MiB)**, SHA-256 `589EAF70252F108A0B717B927793C2203B50B1F82C7FD4CD4BB85B2054B6C37D` (final build). `target/` grew from 19 GB to 24.6 GB (`target/release` 4.1 GB). `speechlab.exe` is 31.4 MB.
- New test tooling: `scripts/ui_test_packaged.mjs` (24 checks on the packaged app; `--empty` for the first start with no model, `--no-mic`, `--install-dir`), `scripts/cdp.mjs` now reads `SPEECHLAB_CDP_MATCH` / `SPEECHLAB_CDP_PORT` (`tauri.localhost` for the packaged app), `scripts/offline_proof_rule.ps1` with three `.cmd` wrappers (firewall rule for the offline proof, run by the owner).
- `docs/MACOS_VALIDATION.md`: the full macOS checklist (prerequisites, sherpa-onnx archive names read in the crate source, whisper.cpp with Metal or Core ML, `externalBin` suffixes, WKWebView differences, `Info.plist` and entitlements, notarisation, file locations, functional checks, result table). Every line NOT VERIFIED.
- `docs/prompts/07-final-report.md`: the prompt for M8.

**Verified (observed on this machine, with the evidence in this entry)**
- *What the installer contains* (silent install to a test folder in 2.1 s, no administrator rights): 7 files, 34,918,850 bytes: `speechlab.exe`, `whisper-cli.exe`, `ggml.dll`, `ggml-base.dll`, `ggml-cpu.dll`, `whisper.dll`, `uninstall.exe`. `whisper-cli.exe` sits next to the executable, so the sidecar lookup of `whisper_cpp.rs` finds it with no code change. No model, voice or `espeak-ng-data` folder is carried.
- *Without any one of the four libraries `whisper-cli.exe` exits with code 127* (tested in an isolated folder); with the four it runs.
- *The window and the lists*: the native window is visible 119 to 132 ms after the process starts (6 starts) and once 265 ms (the first start after a re-install); this is the empty native window, not the content; the model table is rendered after a **median 749 ms (686 to 829 ms, 5 starts, CPU load 22 to 38 %)** measured through the debugging protocol. Not a restart after a reboot (the operating system's file cache was warm). Both model lists show (15 rows) with licences and ratings; the 6 speech-to-text models installed on this machine are listed as installed.
- *Idle memory* (20 s after start, no debugging switch): 8 processes (the application plus 7 web view processes), working set 398 MB in total (shared pages are counted more than once), **private memory 179.5 MB**; `speechlab.exe` alone 29.9 MB working set, 6.6 MB private. Measured on the first build; the CSP build was not measured again.
- *Speech synthesis from the installed app*: the Piper `ljspeech` voice wrote a valid 16-bit 22,050 Hz WAV (this proves that the `espeak-ng-data` folder inside the downloaded voice package is found from the packaged app); one sentence took 836 to 1,829 ms wall time including the voice load (machine at 30 to 66 % CPU load, not a benchmark). Nothing played by itself.
- *Speech-to-text from the installed app*: the generated clip ("The patient takes two tablets every morning.") is transcribed word for word by sherpa-onnx Parakeet TDT v3 int8 and by whisper.cpp base q5_1 (beam search, 5 beams). The process that ran was `whisper-cli.exe` from the install folder (checked by its path). Real-time factors seen (2.7 s clip, loaded machine, NOT benchmark figures): Parakeet 0.12 to 0.36, whisper.cpp base 0.45 to 0.49; Parakeet cold load 2.7 to 6.8 s, whisper.cpp base load 89 to 240 ms.
- *Cancel*: Cancel on a whisper.cpp small transcription ends the `whisper-cli.exe` process (none left 1.5 s later) and re-enables the button; Cancel on a 20-sentence synthesis re-enables Generate and leaves no new audio file. One defect: I-065 (the cancelled transcription shows "operation cancelled" in the error banner).
- *Read-along test on the packaged app*: `scripts/ui_test_readalong.mjs` **33 of 33** (before and after the CSP), including the check that the rewriting box is checked by default for a phonemizer voice (D-043). It also passed 33 of 33 in development with the new `devCsp`.
- *First start with no model* (app started with `SPEECHLAB_MODELS_DIR` pointing at an empty folder): no crash, no error banner, all 15 items "not installed" with an Install button, Transcribe and Generate disabled, the page says models are downloaded once; nothing was written into the empty folder. 11 of 11 checks.
- *Microphone in the release origin (closes I-024 for Windows)*: permission state `prompt`, then the native dialog "http://tauri.localhost souhaite utiliser vos microphones" (Bloquer / Autoriser); after Autoriser (clicked through the debugging protocol on the dialog's own button) the capture works (3 s clip, 16 kHz mono 16-bit) and **the permission is still `granted` after quitting and restarting the app**. A real mouse click was not involved; the choice is stored in the web view profile and applies to the origin `http://tauri.localhost` only.
- *Content Security Policy* (D-045): strict policy in the release build, `devCsp` for development; enforcement verified (an inline script is blocked and raises a `script-src-elem` violation); the first attempt broke the microphone (the AudioWorklet is loaded from a blob URL), which is how `blob:` came into `script-src`. All tests pass with the final policy.
- *Offline proof* (the owner created a firewall rule with `scripts/offline_proof_add.cmd`; it blocks all outbound traffic of the installed `speechlab.exe` and `whisper-cli.exe`; rule listed by `Get-NetFirewallRule` and `netsh`): a model download started from the app failed within 208 ms with "os error 10013" (the block is effective), and with the rule active **both speech-to-text engines and the speech synthesis worked** (19 of 20 checks, the one failure being I-065). Network endpoints of the application's process tree sampled 88 times: no connection from `speechlab.exe` or `whisper-cli.exe`; one established HTTPS connection to `2620:1ec:33::11:443` owned by the web view runtime's network service (I-066). The rule must be removed (`scripts/offline_proof_remove.cmd`, run by the owner).
- *Antivirus and signing*: the installer and executables are unsigned (`NotSigned`); Avast blocked nothing, Defender recorded no detection (I-068).
- *Uninstaller (silent)*: removes the install folder and the uninstall key, keeps the user data (tested with the real data folders renamed away and dummy folders in their place; the real ones were restored, 2580 and 742 files before and after); leaves one registry key (I-069).
- *Development still works*: `cargo build` copies the sidecar and its four libraries next to `target\debug\speechlab.exe`; `cargo test --lib` 163 passed; `pnpm typecheck` and `pnpm test` (16) pass. Without the staged files every cargo build of the package fails (I-064).
- *Visual C++ runtime*: `speechlab.exe` imports only the universal C runtime; the sidecar and its libraries import `MSVCP140`, `VCRUNTIME140`, `VCRUNTIME140_1` and `VCOMP140` (I-070).

**Failed or surprises**
- The sidecar is not one file: four shared libraries are needed (I found it by testing, the plan said "whisper-cli.exe"). They are shipped as resources (D-044); a static rebuild would be cleaner (I-070).
- The owner's first answer to the firewall question said the rule was created, but neither `Get-NetFirewallRule` nor `netsh` showed any rule, and the owner could not copy the command text from the question window: the three `.cmd` files that ask for the administrator prompt by themselves were the fix. Only after the rule was listed did the offline test run.
- My own slips, all harmless: a `bash` command typed in the PowerShell tool started the Windows Subsystem for Linux wrapper and failed ("file or directory" error, nothing of the owner's WSL work was touched); a heredoc left a stray `python3 -` process of mine, which I stopped by its process id; a `node -e` text replacement damaged one line of the new test script (found by `grep` before running, fixed); the first test run used the wrong selector for the "Running..." button and failed on a test defect, not an app defect.
- The first CSP broke the recorder (worklet blob); fixed by `blob:` in `script-src`; a stricter fix (the worklet as a static file) is left for later (D-045).
- `node` prints "Assertion failed: !(handle->flags & UV_HANDLE_CLOSING)" when a DevTools script exits right after closing its socket: tool noise on Windows, no effect on results.
- The installer creates a Desktop shortcut by default (I-069); my test install removed its shortcuts, registry keys and folder afterwards.
- I did not run any whole-network-off test (it would also cut this working session): the proof is a per-program block, and the system web view is not covered (I-066).

**Not verified**
- macOS (Apple Silicon and Intel): nothing built or run; `docs/MACOS_VALIDATION.md` lists every step.
- A clean Windows machine (no developer tools, no Visual C++ redistributable, maybe no WebView2 runtime; the installer's default is to download the WebView2 bootstrapper, which needs the network): not available; whisper.cpp would very probably not start without the redistributable (I-070).
- SmartScreen behaviour for a downloaded installer, code-signing requirements and prices (I-068); the interactive uninstaller (I-069).
- The model download from the packaged app over TLS (the antivirus intercepts TLS here, I-009): only the blocked failure was observed; the download path itself was verified in earlier milestones from the development build.
- Start-up after a reboot (cold file cache); idle memory of the CSP build; memory and speed during a transcription (not a benchmark: use `bench` for figures).
- The microphone dialog answered by a real click; an audio device other than this machine's default.

**Next**
- Owner: remove the firewall rule (`scripts\offline_proof_remove.cmd`), decide on the proposals left open (I-065 fix, I-067 credits view and installer licence page, static whisper.cpp rebuild I-070, the web view's background connection I-066), and run the git commands. Then M8, the final report (`docs/prompts/07-final-report.md`).

**Identity check (end of entry)**: the repository was searched for AI and vendor names; only pre-existing mentions of upstream weights and the Opus codec appear; nothing added by this step. No private content written; `git status` lists no installer, no `target/` output, no `src-tauri/binaries/` file and no model.
