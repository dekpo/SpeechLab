# Git workflow

The owner runs all commit/push/merge commands. Agents only provide them (see `AGENTS.md`). Everything is written in English.

## Conventions

- Default branch: `main`.
- One branch per milestone: `milestone/m0-feasibility`, `milestone/m1-tauri-skeleton`, `milestone/m2-sherpa-stt`, …
- Fixes: `fix/<short-name>`; docs-only: `docs/<short-name>`.
- Commit messages: one short imperative sentence in English, ≤ ~72 chars.
  Good: `Add M0 feasibility report` · `Wire sherpa-onnx offline recognizer` · `Fix WAV resampling to 16 kHz`
- Small, focused commits; the project log is updated in the same commit as the work it describes.
- Never commit: models, build output, audio recordings, secrets, agent-tool folders (see `.gitignore`).

## One-time setup (owner)

```bash
git init -b main
git config user.name  "<your name>"
git config user.email "<your email>"
# Optional remote (replace the URL):
git remote add origin <REMOTE_URL>
```

## M0 — commands to run now

```bash
# 1) Baseline on main
git add Plan.md AGENTS.md README.md .gitignore docs/GIT_WORKFLOW.md
git commit -m "Add project plan and contributor rules"

# 2) M0 work on its own branch
git switch -c milestone/m0-feasibility
git add docs/M0_FEASIBILITY.md docs/PROJECT_LOG.md docs/DECISIONS.md docs/ISSUES.md scripts/check-env.ps1
git commit -m "Add M0 feasibility report and environment check"

# 3) Publish (after the remote exists)
git push -u origin main
git push -u origin milestone/m0-feasibility

# 4) When M0 is accepted, merge into main (locally or via a pull request you open yourself)
git switch main
git merge --no-ff milestone/m0-feasibility -m "Merge milestone M0 feasibility"
git push origin main
```

## M1 — commands to run

M0 must be merged into `main` first (see above). Then:

```bash
git switch main
git switch -c milestone/m1-tauri-skeleton
git add .gitignore README.md package.json pnpm-lock.yaml pnpm-workspace.yaml index.html tsconfig.json vite.config.ts app-icon-source.png
git add src src-tauri docs
git status                      # check: no node_modules, dist, target
git commit -m "Add minimal Tauri 2 app with speech provider contract"
git push -u origin milestone/m1-tauri-skeleton
```

If the files are already modified on top of the M0 branch, `git switch -c milestone/m1-tauri-skeleton` works from there too (uncommitted changes follow you).

## M2 — commands to run

M1 must be merged into `main` first (same merge commands as for M0). Then:

```bash
git switch main
git switch -c milestone/m2-sherpa-stt
git add .gitignore README.md docs scripts src src-tauri
git status                      # check: no wav/, vendor/, models, target, .cargo
git commit -m "Add sherpa-onnx speech-to-text adapter and model manager"
git push -u origin milestone/m2-sherpa-stt
```

If you are still on the M1 branch with these changes uncommitted, `git switch -c milestone/m2-sherpa-stt` carries them over.

## M3 — commands to run

Start from the branch you are on (the work is not committed yet, so do NOT `git switch main` first; the new branch carries the uncommitted changes with it):

```bash
git switch -c milestone/m3-whisper-cpp
git add src-tauri/src/speech/wav.rs
git commit -m "Report the real audio format for non-WAV files"
git add .gitignore README.md docs scripts src src-tauri
git status                      # check: no wav/, vendor/, models, target, .cargo
git commit -m "Add whisper.cpp provider and ggml model support"
git push -u origin milestone/m3-whisper-cpp
```

Then merge everything into `main` with the section "Merging the milestones into `main`" below.

## M4 — commands to run

After M3 is committed, from the branch you are on with the uncommitted M4 files:

```bash
git branch --show-current
git switch -c milestone/m4-audio-compare
git add package.json pnpm-lock.yaml vite.config.ts README.md docs src src-tauri
git status                      # check: no wav/, vendor/, models, target, recordings, .cargo
git commit -m "Add microphone capture, audio import and engine comparison"
git push -u origin milestone/m4-audio-compare
```

## M5a — commands to run

M4 is committed on `milestone/m4-audio-compare`; the M5a files are uncommitted on top of it.

```bash
git branch --show-current
git switch -c milestone/m5-benchmark
git add .gitignore benchmark docs src-tauri
git status                      # check: no benchmark/audio, no samples-private, no wav/, vendor/, target
git commit -m "Add benchmark dataset format, scripts and critical-error detector"
git push -u origin milestone/m5-benchmark
```

## M5b — commands to run

M5a is on GitHub (branch `milestone/m5-benchmark`). The M5b files are uncommitted on top of it, together with the I-031 note. Stay on the same branch:

```bash
git branch --show-current       # milestone/m5-benchmark
git add docs src src-tauri
git status                      # check: no benchmark/audio, no samples-private, no wav/, vendor/, target
git commit -m "Add in-app benchmark dataset recorder"
git push
```

## M5c — commands to run

The M5b commit is done. README.md, the new benchmark code, `benchmark/samples/` (JSON metadata of the 95 recordings, no audio) and `benchmark/results/` (summary and run files, scripted sentences only) are uncommitted. Stay on `milestone/m5-benchmark`:

```bash
git branch --show-current       # milestone/m5-benchmark
git add AGENTS.md README.md docs scripts src src-tauri benchmark
git status                      # check: no benchmark/audio, no samples-private, no results/private, no wav/, vendor/, target, tmp
git commit -m "Add benchmark runner, first results and hand-over documentation"
git push
```

Optionally split it in two commits: first `git add src src-tauri scripts README.md docs` ("Add benchmark runner and scoring fixes"), then `git add benchmark` ("Add recorded dataset metadata and first results").

## Important: commit before you switch branches

`git switch <branch>` is refused if it would overwrite uncommitted changes. If you then run `git merge`, it runs on the branch you are still on and reports "Already up to date". Always commit (or `git stash`) first, check `git branch --show-current`, and only then merge.

## Merging the milestones into `main` (once M0, M1, M2 are committed on their branches)

Each milestone branch contains the previous ones, so merge them in order from `main`:

```bash
git switch main
git branch --show-current                     # must print: main
git merge --no-ff milestone/m0-feasibility    -m "Merge milestone M0 feasibility"
git merge --no-ff milestone/m1-tauri-skeleton -m "Merge milestone M1 Tauri skeleton"
git merge --no-ff milestone/m2-sherpa-stt     -m "Merge milestone M2 sherpa-onnx STT"
git merge --no-ff milestone/m3-whisper-cpp    -m "Merge milestone M3 whisper.cpp"
git push origin main
```

## Pattern for every later milestone

```bash
git switch main
git pull                                   # if a remote is used
git switch -c milestone/mN-short-name
# ... work ...
git status && git diff                     # review
git add <files>
git commit -m "<one short sentence>"
git push -u origin milestone/mN-short-name
# merge into main when accepted
```

Agents will propose the exact `git add` / `git commit -m` lines at the end of each step; run them only if you agree.
