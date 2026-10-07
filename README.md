# AssistantCabinetAI-SpeechLab

Experimental, offline evaluation lab for open-source Speech-to-Text (whisper.cpp, sherpa-onnx) and Text-to-Speech (sherpa-onnx) engines, built as a Tauri 2 + React + TypeScript desktop app. It is a technical proof of concept, **not** a production application, and is independent from the main AssistantCabinetAI repository.

## Status

- M0 feasibility: done.
- M1 minimal Tauri app and provider contract: done.
- M2 sherpa-onnx speech-to-text (model manager, WAV transcription fr/en): done.
- Next: M3 whisper.cpp adapter. See [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md).

## Read first

| File | Purpose |
|---|---|
| [AGENTS.md](AGENTS.md) | Rules for every contributor and agent (git, documentation, constraints) |
| [Plan.md](Plan.md) | Mission, requirements, milestones |
| [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md) | Chronological journal: successes, failures, bugs |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Technical decisions and rationale |
| [docs/ISSUES.md](docs/ISSUES.md) | Open and resolved problems |
| [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md) | Branches, commits, push commands |
| [docs/M0_FEASIBILITY.md](docs/M0_FEASIBILITY.md) | Feasibility and dependency validation |

## Prerequisites (Windows)

Node.js, pnpm, Rust (MSVC toolchain), MSVC Build Tools with the Windows SDK, WebView2. Check with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check-env.ps1
```

CMake and LLVM are only needed later for whisper.cpp (milestone M3).

## Run

```bash
pnpm install
powershell -ExecutionPolicy Bypass -File scripts\fetch-sherpa-libs.ps1   # once; see below
pnpm tauri dev      # starts Vite on port 1430 and opens the native window
```

`scripts/fetch-sherpa-libs.ps1` downloads the prebuilt sherpa-onnx libraries (checksum-verified) and writes a git-ignored `src-tauri/.cargo/config.toml`. It is required only when the automatic download in the crate build fails, typically because an antivirus or proxy intercepts HTTPS (see docs/ISSUES.md I-009). It is harmless otherwise.

## Models and test audio

Models are not in the repository. In the app, use the **Models** table to install one (official download, checksum verified); after that, transcription runs fully offline. They are stored in `%APPDATA%\ai.assistantcabinet.speechlab\models` (override with `SPEECHLAB_MODELS_DIR`). The inventory is `src-tauri/models-manifest.json`.

Put your own WAV files in `wav/` (git-ignored) and type their path in the app. Headless check without the UI:

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

## Layout

```
src/                      React UI; src/speech/ = engine-agnostic TypeScript contract
src-tauri/src/speech/     Rust traits, types, registry, mock provider
src-tauri/src/commands.rs Tauri commands (UI -> Rust)
docs/                     Log, decisions, issues, milestone reports
scripts/                  Helper scripts
```
