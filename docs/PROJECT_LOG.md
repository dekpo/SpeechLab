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
