# M0 — Feasibility and Dependency Validation

Date: 2026-10-06 · Scope: research and environment inspection only. **No application code has been written.**

Status legend: **VERIFIED** = observed in this session (command output / API response) · **NOT VERIFIED** = plausible but not tested here · **NO** = shown not to hold.

Important: "VERIFIED" below means *the artifact exists / the data was read from the official source*. **No engine has been run yet.** Nothing in M0 proves that transcription works, or that it works inside Tauri.

---

## 1. What has been implemented

- `scripts/check-env.ps1` — read-only environment check (reproducible).
- This report.

## 2. Development machine (VERIFIED)

| Item | Value |
|---|---|
| CPU | Intel Core 7 150U — 10 cores / 12 threads |
| RAM | 23.6 GB |
| GPU | Intel integrated graphics (no CUDA-capable GPU → all benchmarks will be **CPU only**) |
| OS | Windows 11 Famille, x64 |
| Free disk | ~642 GB |
| Audio input | Realtek HD Audio, USB audio device, Intel Smart Sound USB audio |
| macOS | **No Mac available → every macOS statement is NOT VERIFIED** |

| Tool | State |
|---|---|
| Node 24.19.0, npm 11.17.0, pnpm 12.4.2 | OK |
| Rust 1.98.1 (`x86_64-pc-windows-msvc`), cargo | OK |
| MSVC Build Tools 14.44 + Windows SDK 10.0.26100 | OK |
| WebView2 runtime 154.x | OK |
| Git 2.55, Python 3.13.14 (+pip) | OK |
| **CMake** | **MISSING** — required to build whisper.cpp / `whisper-rs-sys` |
| **LLVM / libclang** | **MISSING** — required by `bindgen` in `whisper-rs-sys` |
| ffmpeg, ninja | absent (optional) |
| `tauri-cli` | not installed (will be provided via `@tauri-apps/cli` npm package, no global install needed) |
| Network (GitHub) | reachable — needed for downloads only, never for inference |

Blockers to clear before M2: install CMake and LLVM (`winget install Kitware.CMake`, `winget install LLVM.LLVM`). These change the user's system, so they were **not** installed without approval.

## 3. Candidate engines (VERIFIED via GitHub / crates.io APIs)

| | whisper.cpp | sherpa-onnx |
|---|---|---|
| Latest release | v1.9.4 (2026-09-11) | v1.13.8 (2026-09-10) |
| Activity | pushed 2026-10-06 | pushed 2026-10-05 |
| Code license | MIT | Apache-2.0 |
| Prebuilt Windows binaries | **None** in the latest GitHub release (asset list empty) → must be built from source with CMake | Many: `win-x64` shared/static, MD/MT, Release/MinSizeRel, with/without TTS; plus CUDA/DirectML-capable variants |
| Prebuilt macOS binaries | not in release assets | arm64, x64, universal2 (shared/static) |
| Rust binding | `whisper-rs` 0.16.0 (community, Unlicense) → `whisper-rs-sys` 0.15.0, **builds whisper.cpp via cmake + bindgen** (needs CMake + libclang) | `sherpa-onnx` 1.13.8 crate on crates.io, published by the project itself (Apache-2.0), depends on `sherpa-onnx-sys`, which **downloads prebuilt libs at build time** (no CMake/clang needed; features `shared`/`static`) |
| Tauri example upstream | none known | `tauri-examples/` exists upstream (hello_world, file ASR, microphone ASR) — content not inspected |
| Acceleration | CPU, Vulkan, OpenVINO, Core ML, Metal, CUDA (via build flags/features) | CPU (ONNX Runtime); CUDA/DirectML variants exist for Windows |
| Scope | STT (+VAD) | STT, TTS, VAD, diarization, etc. |

Notes:
- Build-time network access (sherpa-onnx-sys download, CMake FetchContent) is **separate from runtime**; this will be documented in the offline/privacy check (M8).
- Tauri stable is **2.12.1**; Tauri 3.0.0-alpha exists. The plan says Tauri 2 → pin to 2.x, ignore 3-alpha.
- `ort` (Rust ONNX Runtime wrapper) is still `2.0.0-rc.13`, i.e. no stable release. We do **not** need it directly; sherpa-onnx brings ONNX Runtime.

## 4. Viable STT models

### whisper.cpp (ggml format, `ggerganov/whisper.cpp` on Hugging Face, license MIT) — VERIFIED listed
tiny / base / small / medium / large-v1/v2/v3 / large-v3-turbo, plus q5/q8 quantized variants. Multilingual (fr + en) except `*.en` variants. Whisper itself is MIT (Apache-2.0 on the `whisper-small` card — to be reconciled in the license table).

### sherpa-onnx (release tag `asr-models`, 498 assets) — VERIFIED listed
| Model | Languages | Size (archive) | Model license |
|---|---|---|---|
| `sherpa-onnx-whisper-{tiny,base,small,medium,large-v3,turbo,distil-*}` | multilingual (`.en` = English only) | 111 MB – 1.8 GB | Whisper = MIT |
| `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8` | multilingual incl. **fr, en** | 465 MB | CC-BY-4.0 |
| `sherpa-onnx-nemo-canary-180m-flash-en-es-de-fr(-int8)` | en, es, de, **fr** | 147 MB (int8) | CC-BY-4.0 |
| `sherpa-onnx-nemo-fast-conformer-{ctc,transducer}-en-de-es-fr-14288(-int8)` | en, de, es, **fr** | ~100 MB (int8) | to verify |
| `sherpa-onnx-streaming-zipformer-fr-2023-04-14` / `-kroko-2025-08-06` | **fr** (streaming) | 380 MB / 55 MB | to verify (Kroko HF card says `other`) |
| `sherpa-onnx-cohere-transcribe-14-lang-int8` | 14 languages | 1.6 GB | Apache-2.0 but **gated** on HF (auto) |
| `sherpa-onnx-qwen3-asr-0.6B-int8` | multilingual | 838 MB | Apache-2.0 |
| `moonshine-*` | en (fr not in the visible list) | 28–240 MB | to verify |

**Key opportunity:** sherpa-onnx can run the *same Whisper weights* as whisper.cpp. That lets M4/M5 separate "runtime difference" from "model difference", which makes the comparison scientifically meaningful (otherwise any gap could be model, quantization, or runtime).

Proposed initial matrix (CPU, French + English):
- whisper.cpp: `base-q5_1`, `small-q5_1`, `large-v3-turbo-q5_0`
- sherpa-onnx: `whisper-small`, `whisper-turbo`, `nemo-parakeet-tdt-0.6b-v3-int8`, `nemo-canary-180m-flash-int8`

Whether the large models are usable on this laptop CPU is **unknown until measured** (RTF is a primary metric).

## 5. Viable TTS models (sherpa-onnx, release tag `tts-models`) — listed VERIFIED, quality NOT VERIFIED

| Model | Language | Voices | Model/dataset license |
|---|---|---|---|
| `vits-piper-fr_FR-siwis-medium` | fr | 1 (SIWIS) | **CC-BY 4.0** |
| `vits-piper-fr_FR-upmc-medium` | fr | 2 | CC-BY-SA 4.0 (share-alike — check implications) |
| `vits-piper-fr_FR-gilles-low` | fr | 1, low quality | CC0 |
| `vits-piper-fr_FR-tom-medium` | fr | 1 | **AGPLv3** → unsuitable for a commercial product |
| `vits-piper-fr_FR-{tjiho-*,miro-high}` | fr | 1 each | model card not found (404) → unknown |
| `vits-mms-fra` | fr | 1 | **CC-BY-NC 4.0 → non-commercial, excluded** |
| `vits-coqui-fr-css10` | fr | 1 | to verify |
| `kokoro-multi-lang-v1_0 / v1_1` (+int8) | multi-language | many | Apache-2.0 for Kokoro-82M; French/English voice list **to verify** |
| `kokoro-int8-en-v0_19`, `matcha-tts-en_US-ljspeech`, `vits-piper-en_*` | en | several | to verify per voice |
| `pocket-tts`, `supertonic-3` | multi (fr listed on pocket-tts card) | few | CC-BY-4.0 (gated) / OpenRAIL |

Findings that already shape the project:
- **French male + female voices are not guaranteed.** Clearly identified French voices: siwis (female, CC-BY), upmc (2 speakers), gilles (low), tom (AGPL). Whether a *commercially usable* male French voice of acceptable quality exists is an **open question** → must be tested in M6.
- **espeak-ng (GPL-3.0)**: sherpa-onnx's Piper/VITS/Kokoro path builds against a fork of espeak-ng for phonemization (`cmake/espeak-ng-for-piper.cmake`; upstream license VERIFIED GPL-3.0). This is a potential **commercial-distribution issue independent of the model license**. Needs legal review and possibly a different TTS path or process isolation. Flagged as a top licensing risk.
- Speed control: sherpa-onnx TTS exposes a `speed` parameter in its generation API; whether it works acceptably per voice is **NOT VERIFIED**.

## 6. Licensing — preliminary (full table in M8)

| Component | License | Commercial use | Flag |
|---|---|---|---|
| whisper.cpp | MIT | yes | attribution |
| whisper-rs | Unlicense | yes | — |
| sherpa-onnx | Apache-2.0 | yes | attribution / NOTICE |
| ONNX Runtime | MIT | yes | attribution |
| Tauri | Apache-2.0 / MIT | yes | — |
| Whisper weights | MIT (OpenAI) | yes | attribution |
| NVIDIA Parakeet / Canary / FastConformer | CC-BY-4.0 | yes | attribution required |
| Piper siwis | CC-BY-4.0 | yes | attribution |
| Piper upmc | CC-BY-SA-4.0 | review | share-alike |
| Piper tom | AGPLv3 | **no (practically)** | exclude |
| MMS-TTS fra | CC-BY-NC-4.0 | **no** | exclude |
| espeak-ng (phonemizer in Piper/Kokoro path) | GPL-3.0 | **high risk** | legal review |
| Cohere Transcribe | Apache-2.0, gated | unclear until terms read | gate |

This is a research summary, **not legal advice**.

## 7. Platform support

| Target | Status |
|---|---|
| Windows 10/11 x64 | sherpa-onnx prebuilt: present (VERIFIED in assets). whisper.cpp: source build, MSVC supported per README. **Neither has been built/run here yet.** |
| macOS Apple Silicon | sherpa-onnx prebuilt arm64/universal2 exist (VERIFIED in assets). whisper.cpp supports Metal/Core ML per README. **NOT VERIFIED — no Mac.** |
| macOS Intel | sherpa-onnx x64 prebuilt exists. **NOT VERIFIED.** |
| Steps to validate on a Mac | install Xcode CLT, Rust, Node, CMake; clone repo; `pnpm i && pnpm tauri build`; run benchmark script; record results in `docs/results/macos-*.json` (to be written in M7) |

## 8. Proposed architecture

```
React UI (TypeScript)
   │  only talks to the contract below
   ▼
Tauri commands + events (JSON, engine-agnostic)
   │
   ▼
Rust core
  ├─ trait SpeechToTextProvider   { capabilities(), load(model), transcribe(audio, opts, cancel), unload() }
  ├─ trait TextToSpeechProvider   { voices(), synthesize(text, voice, speed, cancel) }
  ├─ AudioCaptureService          (WebView getUserMedia → 16 kHz mono PCM → Rust; cpal as fallback)
  ├─ SpeechModelManager           (inventory + download + checksum, separate from inference)
  └─ SpeechBenchmarkService       (runs dataset, WER/CER, RTF, RSS; outputs JSON/CSV)
        ▲                 ▲
  WhisperCppProvider   SherpaOnnxProvider      ← only place that knows an engine
```

Decisions and rationale:
1. **Tauri 2.12 stable**, React + TS + Vite, pnpm. Rust limited to adapter glue — no complex Rust.
2. **sherpa-onnx via the official `sherpa-onnx` crate** (prebuilt download → easy on Windows and macOS).
3. **whisper.cpp: two options, to be decided in M2**:
   - (a) `whisper-rs` in-process — simplest API, but needs CMake + LLVM and its bundled whisper.cpp version must be checked (NOT VERIFIED);
   - (b) build whisper.cpp CLI/server and run as a **Tauri sidecar** — more isolation, trivial cancellation (kill process), no libclang, but IPC and a second binary to package.
   Recommendation: start with (a); fall back to (b) if the build or the version is a problem. The provider trait hides the choice.
4. **Audio capture**: WebView `getUserMedia` first (least native code). Risks: WebView2 and macOS WKWebView microphone permission behaviour inside Tauri — NOT VERIFIED; `cpal` 0.18.2 is the fallback.
5. **Cancellation**: cooperative flag where the engine exposes a callback (whisper.cpp abort callback), otherwise run inference in a worker thread/process that can be dropped/killed. Whether sherpa-onnx offline recognition can be interrupted mid-decode is NOT VERIFIED.
6. **Integration boundary with AssistantCabinetAI**: the future app consumes only the Rust traits plus the Tauri command contract (a separate crate `speech-core`); engines and models stay behind adapters. Nothing in this repo touches the main repository.
7. **Models are never bundled in the installer**; downloaded on demand from official sources into the app data directory, with SHA-256 pinned in the model manifest.

## 9. Short implementation plan (next milestones)

| Milestone | Content | Exit criterion |
|---|---|---|
| M1 | Scaffold Tauri 2 + React/TS; define the provider contract with a **mock** provider (clearly labelled, not used in benchmarks) | `pnpm tauri dev` opens window on Windows |
| M2 | **sherpa-onnx adapter (STT)** + model manager download (order swapped, see DECISIONS D-002) | transcribe a WAV fr+en offline |
| M3 | **whisper.cpp adapter** (decide a vs b; needs CMake, + LLVM for option a) | same WAV, same contract |
| M4 | Mic capture, WAV import, comparison UI | both engines on a recording |
| M5 | Dataset format + WER/CER + critical-error flags + RTF/RSS | reproducible `bench` run → JSON/CSV |
| M6 | TTS lab | French/English voices, speed, WAV export |
| M7 | Packaging validation (Windows built; macOS = steps only) | installer, startup, DLL placement |
| M8 | `docs/SPEECH_ENGINE_EVALUATION.md`, license table, recommendation | report from measured data only |

## 10. Verified / unverified / limitations

**Verified:** machine specs; installed tools; missing CMake and LLVM; GitHub/crates.io versions and licenses; existence of prebuilt sherpa-onnx Windows/macOS libs; existence of French STT and TTS model archives; model/dataset licenses read from official cards; espeak-ng is GPL-3.0 and used in sherpa-onnx's Piper path.

**Not verified:** that any engine builds or runs here; French/English accuracy; TTS naturalness; whether a free-to-use French male voice exists; whisper-rs's bundled whisper.cpp version; microphone access inside Tauri WebViews; all macOS behaviour; Swiss-French test data availability (no source identified yet — Common Voice French has accent metadata but its coverage of Suisse romande is unchecked); cancellation support in sherpa-onnx.

**Known limitations of M0:** API/metadata inspection only; license reading is informal; model-card 404s (tjiho, miro) leave some voices undocumented.

## 11. Recommended next step

1. Approve installing **CMake** and **LLVM** (or choose sherpa-onnx-first and defer whisper.cpp tooling).
2. Start **M1** (minimal Tauri 2 app + provider contract).
3. In parallel, decide the licensing stance on **espeak-ng (GPL-3.0)** because it can eliminate Piper/Kokoro as commercial TTS options.
