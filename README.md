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
- Next: owner checks one suspect sample (I-033), then long-audio chunking, drug-name correction, accent clips.

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
```

## Models and test audio

Models are not in the repository. In the app, use the **Models** table to install one (official download, checksum verified); after that, transcription runs fully offline. They are stored in `%APPDATA%\ai.assistantcabinet.speechlab\models` (override with `SPEECHLAB_MODELS_DIR`). The inventory is `src-tauri/models-manifest.json`.

In the app you can record from the microphone (Windows asks for permission the first time) or import an audio file (WAV, MP3, M4A, Ogg/Opus). It is converted to 16 kHz mono WAV and stored locally in `%APPDATA%i.assistantcabinet.speechlab
ecordings`, with a Delete button. You can also keep your own WAV files in `wav/` (git-ignored) and type their path. Headless check without the UI:

```bash
cd src-tauri
cargo run --example transcribe -- list
cargo run --example transcribe -- install sherpa-canary-180m-flash-int8
cargo run --example transcribe -- run sherpa-canary-180m-flash-int8 fr ../wav/sample.wav 3
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
cargo run --release --example bench -- rescore --dir ../benchmark/results/<folder>   # apply improved scoring rules
python ../scripts/bootstrap_ci.py ../benchmark/results/<folder>    # confidence intervals
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
