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
