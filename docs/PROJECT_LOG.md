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
