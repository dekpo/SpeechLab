# AGENTS.md — Project rules (read first, every session)

These rules apply to **every** person and automated agent working on this repository, in every session. Read this file, then `docs/PROJECT_LOG.md`, before doing anything.

Project: **AssistantCabinetAI-SpeechLab** — an experimental, offline STT/TTS evaluation lab. See `Plan.md` for the mission and `docs/` for status. It is NOT a production app and must never modify the main AssistantCabinetAI repository.

## 1. Git rules (mandatory)

1. **Never run `git commit`, `git push`, `git merge`, `git rebase`, `git tag`, or create/merge a pull request without the owner's explicit prior approval.** The owner runs these commands personally.
2. Agents MAY run read-only git commands (`status`, `diff`, `log`, `branch --list`).
3. At the end of each step, the agent **provides the exact git commands** (branch, add, commit, push) for the owner to run. See `docs/GIT_WORKFLOW.md` for conventions.
4. Branch per milestone/feature: `milestone/mN-short-name` or `fix/short-name`. `main` only receives reviewed, working states.
5. Commit messages: **English, one short sentence**, imperative mood, no trailing period required, max ~72 characters. Example: `Add sherpa-onnx STT adapter`.
6. Commit messages, PR descriptions, code, comments, and documentation are all **in English**.

## 2. Identity rule (mandatory)

**Never mention any AI assistant, model, vendor, or related website/URL** — not in commits, commit trailers (no `Co-Authored-By`), PR text, code, comments, documentation, file names, configuration, or generated reports. Do not add "generated with…" lines. Tool-specific folders (e.g. an agent's settings directory) are git-ignored and must not be committed.

## 3. Documentation rules (mandatory)

Documentation is what lets work continue across many sessions and agents. All documentation is **in English**.

- After every meaningful step, update `docs/PROJECT_LOG.md` (append-only, newest at the bottom): what was done, commands used, **results, successes, failures, bugs, dead ends, and why**. Failures are as valuable as successes — never delete them.
- Record every non-trivial technical choice in `docs/DECISIONS.md` (context, options, decision, consequences).
- Record every bug/problem in `docs/ISSUES.md` (symptom, cause, status, fix or workaround).
- At the end of each milestone, write its summary using the five headings: **Implemented / Verified / Not verified / Known limitations / Next step**.
- Mark every claim **VERIFIED** (observed in this repo/machine, with evidence) or **NOT VERIFIED** (with the exact steps to verify). Never present an assumption as a result.
- Keep `README.md` current with how to install and run.

## 4. Engineering constraints (from `Plan.md`)

- Prefer **free and open-source** tools, libraries, and models. Any non-free or restricted component must be flagged and justified in `docs/DECISIONS.md`.
- Reject models/dependencies whose license forbids commercial use (e.g. CC-BY-NC) or is copyleft-incompatible without explicit owner approval (e.g. AGPL, GPL).
- Everything must run **locally/offline** at inference time. Network use is allowed only for explicit, separate downloads (models, build dependencies).
- No simulated or invented benchmark data. No real patient data. Test audio must be synthetic, public-domain, or properly licensed.
- Correctness over visual polish. Keep dependencies minimal. No hardcoding a single engine in the UI; go through the provider interfaces.
- Do not install software or change the system without the owner's approval; propose the command instead.
- Do not auto-send transcripts to an AI agent; do not auto-play TTS.
- Do not skip a milestone silently. If order changes, record it in `docs/DECISIONS.md`.

## 5. Session start / end checklist

Start: read `AGENTS.md` → `docs/HANDOFF.md` (state, traps, commands, backlog) → `docs/PROJECT_LOG.md` (last entries) → `docs/ISSUES.md` (open items) → `git status`/`git log` (read-only) and `tasklist` (is a benchmark running?). Ready-made prompts for new sessions are in `docs/prompts/`.
End: update log/decisions/issues and `docs/HANDOFF.md` sections 3 and 7 → give the owner the git commands for the step (check `git status` so no path is forgotten) → state the next step.

## 6. Parallel sessions and long jobs

- Several sessions may exist over time, but two sessions must not edit the same files at once, and **never run two benchmarks at once**.
- A benchmark needs a quiet machine: run `bench preflight` first; do not compile, test or record while it runs; never publish figures from a forced or disturbed run.
- A long job (benchmark) keeps running if the session is interrupted. In a new session, check `tasklist` and the job's log before starting anything.
- Never touch the owner's other processes (their other app on port 1420, Docker, WSL): ask the owner to close them.

## 7. SpeechLab is frozen; the speech integration into AssistantCabinetAI is ON HOLD (D-048, D-050)

Read `docs/integration/STATUS.md` at the start of **every** session. State on 2026-10-09:

- **SpeechLab is finished and frozen** (M0 to M8 delivered, `docs/SPEECH_ENGINE_EVALUATION.md`). No new feature. Reopen it only to measure or for the integration-preparation phase SP-0. **macOS is on hold** until further notice (no Mac).
- The owner will **first finish the Knowledge Base (KB) in AssistantCabinetAI**, **then relaunch a pass on the integration plan to adapt it** (tab "Speech R — Re-baseline the plan", `docs/prompts/integration/SP-R-rebaseline.md`). Until `docs/integration/STATUS.md` says `STATUS: RESUMED`, **do not run any `SP-*` launcher** (except SP-0 if the owner explicitly asks) and do not treat `docs/integration/01..03` as current facts about AssistantCabinetAI: they are draft v1 of 2026-10-09.
- Never modify the AssistantCabinetAI repository from here (rule above). Reading it is allowed.
- Only the owner changes the status line. An agent that runs the re-baseline proposes the change and waits for the owner's explicit yes.
- If a request from the owner touches speech integration while the status is ON HOLD, say so in French and ask whether to run the re-baseline pass first.
