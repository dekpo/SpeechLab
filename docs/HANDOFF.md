# HANDOFF — read this right after `AGENTS.md`

For any agent or person starting a new session. It tells you where the project stands, what the
traps are, how to run things, and what to do next. Keep it up to date: when you finish a step,
change the "Where things stand" and "Backlog" sections in the same commit as your work.
All repository text is English. The owner reads and writes French: answer the owner in French in
the chat, write files in English.

## 1. The project in five lines

- **AssistantCabinetAI-SpeechLab**: an experimental, offline lab that evaluates open-source
  speech-to-text (whisper.cpp, sherpa-onnx) and text-to-speech engines, to decide what to
  integrate later into AssistantCabinetAI (a separate product, never touched from here).
- Desktop app: Tauri 2 + React + TypeScript + Rust. Engines run locally, CPU only on the dev machine.
- Mission and constraints: `Plan.md`. Rules for everyone: `AGENTS.md`. Decisions: `docs/DECISIONS.md`.
  Problems: `docs/ISSUES.md`. Chronological journal: `docs/PROJECT_LOG.md` (read its last entries).
- Goal: measured, reproducible evidence (accuracy, critical errors, speed, memory, licences), not
  opinions. Nothing simulated, nothing extrapolated, every claim marked verified or not verified.
- Final deliverable: `docs/SPEECH_ENGINE_EVALUATION.md` (milestone M8).

## 2. Rules that get broken most often (full text in `AGENTS.md`)

1. **Never run `git commit`, `git push`, `git merge`, `git tag` or open a PR.** Give the owner the exact commands instead (branch, add, one-sentence English commit, push). Read-only git is fine.
2. **Never mention any AI assistant, model, vendor or related URL** anywhere in the repository (code, docs, commits, file names, config). No "Co-Authored-By", no "generated with".
3. **Write down everything in `docs/`**: successes, failures, dead ends, surprises, in the log. Mark each claim VERIFIED (observed here, with evidence) or NOT VERIFIED (with the steps to verify).
4. **Free and open-source first**; reject non-commercial or copyleft models/libraries without the owner's approval (known flag: espeak-ng is GPL-3.0, see D-012).
5. **Do not install software or change the system without the owner's approval.** Propose the command.
6. **No private content in the repository or in documents**: voices of third parties, the owner's voice notes, the first name of the addressee of a private message, personal data. Only aggregate numbers.

## 3. Where things stand

| Milestone | Status |
|---|---|
| M0 feasibility | done, merged |
| M1 Tauri skeleton and provider contract | done, merged |
| M2 sherpa-onnx STT (model manager, downloader, UI) | done, merged |
| M3 whisper.cpp STT (external `whisper-cli` process, real cancellation) | done, merged |
| M4 microphone, audio import, clip store, engine comparison view, WER/CER | done, committed |
| M5a dataset format, 95 reading scripts, number folding, critical-error detector | done, pushed |
| M5b in-app dataset recorder | done |
| M5c benchmark runner, full benchmark | done: first run (disturbed) plus clean re-run `20261007-215116-full-owner-reps1-clean` (855/855 transcripts identical; speed figures measured at about 20 % background CPU, D-034). Variance still unmeasured (1 repetition) |
| M6 TTS laboratory | not started |
| M7 Windows/macOS packaging validation | not started (no Mac available: macOS stays NOT VERIFIED, document the steps) |
| M8 final report, licensing table, recommendation | not started |

Check `git log --oneline -5` and `git status` (read-only) to see what the owner has committed.

**What the benchmark says (one speaker, CPU, 1 repetition, speed at about 20 % background CPU)**:
sherpa-onnx Parakeet TDT v3 int8 has the lowest WER (2.3 %) at RTF 0.11, Canary int8 is comparable
on clean input but once produced garbage, whisper.cpp small is accurate (3.9 %) but still slower
than real time on this CPU (RTF 1.1 to 1.2), tiny/base Whisper are too inaccurate. Every engine
misspells drug names. Accuracy is load-independent (clean re-run identical to the disturbed run).
Details and caveats: last M5c entries of `docs/PROJECT_LOG.md`, results in `benchmark/results/`.
Provisional shortlist: D-032 (not final).

## 4. Repository map

```
Plan.md, AGENTS.md           mission and rules
docs/                        PROJECT_LOG, DECISIONS, ISSUES, GIT_WORKFLOW, HANDOFF, M0 report, prompts/
benchmark/scripts/           95 sentences to read aloud (committed)
benchmark/samples/           metadata + reference for each recorded sample (committed, no audio)
benchmark/samples-private/   same, for third-party voices (git-ignored, never quote)
benchmark/audio/             the WAV recordings (git-ignored)
benchmark/results/<run>/     benchmark outputs (summary.md, summary.csv, runs.jsonl, system.json, config.json)
src/                         React UI (components, audio capture, engine-agnostic TypeScript contract)
src-tauri/src/speech/        Rust: provider traits, sherpa.rs, whisper_cpp.rs, models.rs, download.rs, clips.rs,
                             dataset.rs, metrics.rs, numbers.rs, critical.rs, probe.rs, benchmark.rs
src-tauri/examples/          CLI tools: transcribe.rs (one file), bench.rs (benchmark)
src-tauri/models-manifest.json   model inventory (data, no code change to add a model)
scripts/                     PowerShell/Python helpers (env check, library/CLI builds, comparison, bootstrap CI)
vendor/, wav/, models, target/   git-ignored (downloads, builds, the owner's private audio)
```

## 5. Environment facts and traps (Windows 11, Intel Core 7 150U, 23.6 GB, CPU only)

- **Dev server port is 1430**, not 1420: 1420 belongs to the owner's separate AssistantCabinetAI app. Never stop that process.
- **TLS interception by an antivirus** breaks the sherpa-onnx crate build download (`UnknownIssuer`). Run `scripts/fetch-sherpa-libs.ps1` once; it writes the git-ignored `src-tauri/.cargo/config.toml`. Model downloads work (native certificates). Never disable the antivirus or TLS checks.
- **whisper.cpp** is built from source by `scripts/build-whisper-cpp.ps1` into `vendor/` (CMake required; LLVM is not).
- **Models** live in `%APPDATA%\ai.assistantcabinet.speechlab\models` (all installed except whisper turbo). Clips in `...\recordings`.
- **Benchmarks need a calm machine.** The owner also runs Docker Desktop/WSL, which uses CPU. On this laptop the Windows firewall engine, DNS client and the security software alone keep the background load at 16 to 31 %, so `bench run` refuses to start above **30 %** average CPU or on battery (D-034, relaxed from 15 % by the owner's choice to mimic a client machine). Run `bench preflight` first and quote the recorded load with any speed figure. Never use `--force` for figures you intend to publish. Do not compile, test, record or take screenshots while a benchmark runs.
- **A running benchmark survives an interrupted session.** Before starting another one, check `tasklist` for `bench.exe`/`whisper-cli.exe` and the log file. Never run two benchmarks at the same time.
- **Use `--release`** for any speed or memory figure.
- **GitHub once returned HTTP 500** when creating a new branch from the M4 commit (I-031); creating the branch from `main` in the web UI worked, then a normal push.
- **Shell quoting**: very long inline shell/Python scripts containing apostrophes were rejected by the agent shell several times. Write script files and run them, or use small edits. In Python strings avoid backslash escapes such as `\b`.
- **Read generated test code** before running it, and validate any scoring rule on real engine output before trusting a number (five scoring defects were found this way, see PROJECT_LOG and D-030).
- UI testing without a human: run the app with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` and drive the page over the DevTools protocol (select the target whose url contains `localhost:1430`; the microphone permission prompt is a separate target; grant it with `Browser.grantPermissions`). Clean up any sample or clip created by tests.

## 6. How to run things

```bash
pnpm install
pnpm typecheck && pnpm test && pnpm build          # frontend checks
cd src-tauri && cargo test --lib                    # Rust unit tests (about 77)
pnpm tauri dev                                      # the app (port 1430)
powershell -ExecutionPolicy Bypass -File scripts/check-env.ps1     # machine and tool check

cd src-tauri
cargo run -q --example transcribe -- list           # models and install state
cargo run -q --example transcribe -- run <model-id> <fr|en> <file.wav> [repeat]

cargo build --release --example bench
./target/release/examples/bench.exe check           # dataset and audio checks
./target/release/examples/bench.exe preflight       # quiet machine? exit 0 yes, 2 no
./target/release/examples/bench.exe run --reps 1 --label <name>
./target/release/examples/bench.exe rescore --dir ../benchmark/results/<run>    # apply newer scoring rules
./target/release/examples/bench.exe summarize --dir ../benchmark/results/<run>

python ../scripts/bootstrap_ci.py ../benchmark/results/<run> [--exclude en-it-05-owner]
python ../scripts/compare_runs.py ../benchmark/results/<old> ../benchmark/results/<new>
```

## 7. Backlog (priority order; ready-made prompts in `docs/prompts/`)

| # | Task | Why | Prompt |
|---|---|---|---|
| 1 | ~~Clean re-run of the full benchmark~~ DONE 2026-10-07 (I-039 resolved) | | `docs/prompts/01-clean-benchmark-rerun.md` (kept for reference) |
| 2 | Owner listens to `en-it-05`, re-record if needed, re-run only that sample | all engines hear "543", script says "443" (I-033) | in next-tasks.md |
| 3 | **Next:** timing study, 3 repetitions on a subset, background CPU logged before and after | variance (I-012) | next-tasks.md T2 |
| 4 | Long audio: VAD chunking | truncation, loops (I-017, I-018, I-019, I-029) | next-tasks.md T3 |
| 5 | Drug-name handling (hotwords, prompts, dictionary correction) | every engine misspells drug names | next-tasks.md T4 |
| 6 | Private accent clips (Swiss-Romande, Maghreb) as long private samples | accents are in the plan | next-tasks.md T5 |
| 7 | Detector false alarms for times ("10.30", "9h00"), sherpa-onnx thread usage | I-034, I-036 | small fixes |
| 8 | M6 TTS laboratory | plan section 5C | next-tasks.md T6 |
| 9 | M7 packaging validation | plan section 9 | next-tasks.md T7 |
| 10 | M8 final report and licensing table | main deliverable | next-tasks.md T8 |

## 8. Session protocol

Start: `AGENTS.md` → this file → last entries of `docs/PROJECT_LOG.md` → open items in `docs/ISSUES.md` → `git status` and `git log` (read-only) → check no benchmark is running (`tasklist`).

During: small steps, tests with every change, document failures as you go.

End:
1. Append a dated entry to `docs/PROJECT_LOG.md` using the headings **Done / Verified / Failed or surprises / Not verified / Next**.
2. Update `docs/DECISIONS.md`, `docs/ISSUES.md` and, if the state changed, sections 3 and 7 of this file and `README.md`.
3. Check the identity rule: search the repository for AI or vendor names (none must appear) and make sure no private content was written.
4. Give the owner the exact git commands (branch, `git add` of explicit paths, one-sentence English commit, push) and **do not run them**. Check `git status` first so the list is complete (earlier hand-offs forgot `README.md`).
5. Tell the owner, in French: what was done, what is verified, what is not, limits, and the proposed next step.

## 9. Vocabulary

- **WER / CER**: word / character error rate. Micro WER = total errors over total reference words.
- **RTF**: real-time factor = processing time / audio duration (below 1 is faster than real time).
- **Cold start**: the run that includes loading the model. whisper.cpp runs as a new process each time, so it is always cold.
- **Decoding**: greedy (1 candidate) or beam search (several). Always report which one was used (D-018).
- **Critical flag**: a changed, lost or added number, unit, negation or weekday/month, or a missing key term (drug names are critical). Heuristic, about 15 % false alarms on times; keep expected/found text for human review.
- **Scoring version**: scoring rules are versioned; old results can be re-scored from stored transcripts (D-030).
- **Suspect sample**: most engines agree with each other but not with the reference: the reading or the reference may be wrong (I-033).
