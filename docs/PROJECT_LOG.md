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
