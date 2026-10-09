# AssistantCabinetAI-SpeechLab

Experimental, offline evaluation lab for open-source Speech-to-Text (whisper.cpp, sherpa-onnx) and Text-to-Speech (sherpa-onnx) engines, built as a Tauri 2 + React + TypeScript desktop app. It is a technical proof of concept, **not** a production application, and is independent from the main AssistantCabinetAI repository.

## Status

- M0 feasibility: done.
- M1 minimal Tauri app and provider contract: done.
- M2 sherpa-onnx speech-to-text (model manager, WAV transcription fr/en): done.
- M3 whisper.cpp speech-to-text (external process, ggml models, real cancellation): done.
- M4 microphone capture, audio import (WAV/MP3/M4A/Ogg), local clip store, engine comparison with WER/CER and diff: done.
- M5a benchmark dataset format, 95 reading scripts, spoken-number folding and critical-error detector: done (see `benchmark/README.md`).
- M5b in-app dataset recorder (read a sentence, record, save with its reference): done.
- M5c reproducible benchmark runner, first full benchmark of 9 configurations on 95 recorded sentences: done (results in `benchmark/results/`, analysis in [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md)). A clean re-run (`20261007-215116-full-owner-reps1-clean`, about 20 % background CPU) reproduced all 855 transcripts identically and replaced the speed figures of the first, disturbed run; a 3-repetition timing study (`20261008-035938-timing-3reps`, M5d) then showed that the speed of the same sample varies by only 1 to 4 % within a run and that no transcript changes between repetitions.
- M5e long audio: engine-independent chunking at silences with the Silero VAD (`bench run --chunking vad`, default off). Short audio is provably unchanged; chunking helps only sherpa-onnx Whisper tiny (its 30 s limit) and hurts Canary; Whisper loops remain (D-035, project log).
- T4 drug names and key terms (D-036): vocabulary biasing (whisper.cpp initial prompt, Parakeet hotwords) and dictionary post-correction, all OFF by default. Strict post-correction fixed more drug names with no word broken; loose settings and a high hotword score are regressions. One speaker, a vocabulary taken from the test sentences: an upper bound.
- M6 text-to-speech laboratory (D-038): a TTS tab (language, voice, speed, generate, then play/stop/save WAV; nothing plays automatically) over sherpa-onnx with Piper and Kokoro voices installed from the app, plus the `tts` command line tool. Every voice is rated for commercial use (`docs/TTS_LICENSES.md`, D-039); Piper and Coqui voices run 16 to 29 times faster than real time on this CPU, Kokoro int8 slower than real time; naturalness awaits the owner's listening notes (`python -I -X utf8 scripts/tts_listening.py` builds the listening page). The phonemizer inside these voices is espeak-ng (GPL-3.0), a licensing point for any product use.
- M6c: the owner's blind listening is recorded in `docs/TTS_LISTENING_NOTES.md`; decisions D-040 (sentence highlight with auto-scroll) and D-041 (beta only with freely usable voices) taken.
- M6d (D-042): the TTS tab splits the text into sentences, synthesises them one by one into a single WAV with measured sentence times, and after Generate shows the text with the sentence being read shaded in grey (light and dark, automatic scrolling, "Follow reading", nothing plays by itself); a text normaliser (numbers, dates, times, units, abbreviations, French and English) makes the character-based French voice speak digits and can be switched per voice, with a preview of the text sent to the voice; Piper `kristin` measured; Piper `mls` French voice converted locally (`scripts/convert_piper_voice.py`); Chatterbox and Qwen3-TTS read and not retained. Blind listening session 2: `python -I -X utf8 scripts/tts_listening.py --session 2`.
- M7 Windows packaging (D-044, D-045): per-user NSIS installer (8.45 MiB) with `whisper-cli.exe` as a sidecar plus its four libraries, no model bundled, strict Content Security Policy; installed and tested on this machine (window, model lists, speech-to-text with both engines, speech synthesis, cancel, microphone in the release origin, first start without models, offline proof for the application's own processes). Unsigned; clean-machine behaviour, SmartScreen and macOS are NOT VERIFIED (`docs/MACOS_VALIDATION.md`, issues I-064 to I-070).
- Next: M8, the final report and licensing table (`docs/prompts/07-final-report.md`); owner checks one suspect sample (I-033).

## Read first

| File | Purpose |
|---|---|
| [AGENTS.md](AGENTS.md) | Rules for every contributor and agent (git, documentation, constraints) |
| [Plan.md](Plan.md) | Mission, requirements, milestones |
| [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md) | Chronological journal: successes, failures, bugs |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Technical decisions and rationale |
| [docs/ISSUES.md](docs/ISSUES.md) | Open and resolved problems |
| [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md) | Branches, commits, push commands |
| [docs/HANDOFF.md](docs/HANDOFF.md) | State of the project, traps, commands and backlog for any new session |
| [docs/MACOS_VALIDATION.md](docs/MACOS_VALIDATION.md) | macOS checklist (everything NOT VERIFIED: no Mac available) |
| [docs/prompts/](docs/prompts/README.md) | Ready-made prompts to continue in a new chat session |
| [docs/M0_FEASIBILITY.md](docs/M0_FEASIBILITY.md) | Feasibility and dependency validation |

## Prerequisites (Windows)

Node.js, pnpm, Rust (MSVC toolchain), MSVC Build Tools with the Windows SDK, WebView2. Check with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check-env.ps1
```

For whisper.cpp you also need CMake (`winget install Kitware.CMake`). LLVM is not needed.

## Run

```bash
pnpm install
powershell -ExecutionPolicy Bypass -File scripts\fetch-sherpa-libs.ps1   # once; see below
pnpm tauri dev      # starts Vite on port 1430 and opens the native window
```

`scripts/fetch-sherpa-libs.ps1` downloads the prebuilt sherpa-onnx libraries (checksum-verified) and writes a git-ignored `src-tauri/.cargo/config.toml`. It is required only when the automatic download in the crate build fails, typically because an antivirus or proxy intercepts HTTPS (see docs/ISSUES.md I-009). It is harmless otherwise.

To use the whisper.cpp engine, build its CLI once (clones the official repository at a pinned tag into the git-ignored `vendor/` and compiles it, about 2 minutes):

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-whisper-cpp.ps1
powershell -ExecutionPolicy Bypass -File scripts\stage-whisper-sidecar.ps1   # required for every cargo build, test and tauri dev/build (I-064)
```

The second command copies `whisper-cli.exe` and its four libraries into the git-ignored `src-tauri/binaries/`, where the Tauri bundler expects the sidecar (D-044). Without it the Rust build of the app stops with "resource path ... doesn't exist".

## Build and install the Windows package

```powershell
powershell -ExecutionPolicy Bypass -File scripts\fetch-sherpa-libs.ps1    # once, only if the crate download fails (antivirus)
powershell -ExecutionPolicy Bypass -File scripts\build-whisper-cpp.ps1    # once
powershell -ExecutionPolicy Bypass -File scripts\stage-whisper-sidecar.ps1
pnpm install
pnpm tauri build                                                          # 2 to 3 minutes once the dependencies are compiled
```

The installer is `src-tauri\target\release\bundle\nsis\SpeechLab_0.1.0_x64-setup.exe` (about 8.5 MiB, git-ignored, unsigned). It installs per user, with no administrator rights (VERIFIED with a silent install to a chosen folder; the default folder, expected to be under `%LOCALAPPDATA%`, was not observed), plus a Desktop and a Start Menu shortcut. **It carries no model**: the first start shows every model as "not installed" with an Install button; models, recordings and generated speech stay in `%APPDATA%\ai.assistantcabinet.speechlab` and survive an uninstall. A silent install to a chosen folder, for tests: `SpeechLab_0.1.0_x64-setup.exe /S /D=<folder>` (the folder must be the last argument).

Tests of the installed app (start it with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` and `SPEECHLAB_CDP_MATCH=tauri.localhost`, then `node scripts/ui_test_packaged.mjs --install-dir <folder>` or `node scripts/ui_test_readalong.mjs`). Start it with `SPEECHLAB_MODELS_DIR=<empty folder>` and add `--empty` to test the first start. macOS: `docs/MACOS_VALIDATION.md` (not verified).

## Models and test audio

Models are not in the repository. In the app, use the **Models** table to install one (official download, checksum verified); after that, transcription runs fully offline. They are stored in `%APPDATA%\ai.assistantcabinet.speechlab\models` (override with `SPEECHLAB_MODELS_DIR`). The inventory is `src-tauri/models-manifest.json`.

In the app you can record from the microphone (Windows asks for permission the first time) or import an audio file (WAV, MP3, M4A, Ogg/Opus). It is converted to 16 kHz mono WAV and stored locally in `%APPDATA%i.assistantcabinet.speechlab
ecordings`, with a Delete button. You can also keep your own WAV files in `wav/` (git-ignored) and type their path. Headless check without the UI:

```bash
cd src-tauri
cargo run --example transcribe -- list
cargo run --example transcribe -- install sherpa-canary-180m-flash-int8
cargo run --example transcribe -- run sherpa-canary-180m-flash-int8 fr ../wav/sample.wav 3
cargo run --example transcribe -- install silero-vad                      # support model for chunking (0.6 MB, MIT)
cargo run --example transcribe -- run sherpa-parakeet-tdt-0.6b-v3-int8 fr ../wav/long.wav 1 --chunking vad
```

Other commands:

```bash
pnpm typecheck                 # TypeScript check
pnpm build                     # production frontend build
cd src-tauri && cargo test     # Rust unit tests
```

The dev server uses port **1430** on purpose (1420 is used by the AssistantCabinetAI desktop app). The first Rust build takes several minutes.

## Benchmark

```bash
cd src-tauri
cargo run --release --example bench -- check                       # dataset and audio checks
cargo run --release --example bench -- run --reps 1 --label mine   # every installed model, both whisper.cpp decodings
cargo run --release --example bench -- run --models sherpa-parakeet-tdt-0.6b-v3-int8 --category fr-general --reps 3
cargo run --release --example bench -- run --category fr-dictation,en-dictation --chunking vad --label long-vad   # cut clips over 25 s at silences
cargo run --release --example bench -- rescore --dir ../benchmark/results/<folder>   # apply improved scoring rules
cargo run --release --example bench -- run --vocab-dir ../benchmark/vocab --models whisper-cpp-small-q5_1 --label prompt   # initial prompt from benchmark/vocab/<lang>.txt
cargo run --release --example bench -- run --vocab-dir ../benchmark/vocab --hotwords-score 1.5 --models sherpa-parakeet-tdt-0.6b-v3-int8 --label hotwords   # Parakeet hotwords
cargo run --release --example bench -- postcorrect --dir ../benchmark/results/<folder> --vocab-dir ../benchmark/vocab --preset strict --label pc   # dictionary correction of stored transcripts
cargo run --release --example bench -- termstudy --dir ../benchmark/results/<baseline> --against ../benchmark/results/<variant>   # key terms found, fixed versus broken words
cargo run --release --example tts -- voices                                   # text-to-speech: installed voices
cargo run --release --example tts -- say <voice-id> fr "Bonjour." --speed 1.0   # writes a WAV, never plays it
cargo run --release --example tts -- say <voice-id> fr "@file.txt" --show-text 1 --show-segments 1   # sentences, text sent to the voice, segment times (--normalise on|off, --whole 1)
cargo run --release --example tts -- measure --sentences ../benchmark/tts/paragraphs-fr.txt --lang fr --voices <id> --reps 5 --compare 1 --out-dir ../benchmark/results/<run> --label x   # per-sentence overhead
python -I -X utf8 ../scripts/tts_listening.py --session 2   # blind listening page (benchmark/tts-samples/session2/index.html)
node ../scripts/ui_test_readalong.mjs   # read-along UI test; start the app first with WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 pnpm tauri dev
python ../scripts/bootstrap_ci.py ../benchmark/results/<folder>    # confidence intervals
python ../scripts/chunking_study.py runs ../benchmark/results/<whole> ../benchmark/results/<chunked>   # whole clip versus chunked
```

Results go to `benchmark/results/<UTC stamp>-<label>/` (`summary.md`, `summary.csv`, `runs.jsonl`, `system.json`, `config.json`). Always use `--release` for speed figures and avoid heavy work on the machine during a run. `bench run` refuses to start above 30 % average background CPU or on battery (D-034); quote the recorded load (`config.json`, `preflight.idleCpuPercent`) with any speed figure. See `benchmark/README.md` for the dataset format.

## Layout

```
src/                      React UI; src/speech/ = engine-agnostic TypeScript contract
src-tauri/src/speech/     Rust traits, types, registry, mock provider
src-tauri/src/commands.rs Tauri commands (UI -> Rust)
docs/                     Log, decisions, issues, milestone reports
scripts/                  Helper scripts
```
