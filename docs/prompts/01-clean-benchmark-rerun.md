# Prompt 01 — Clean re-run of the full benchmark

Paste everything below the line into a new chat session. The owner must have stopped Docker
Desktop / WSL, plugged in the charger, and agreed to leave the computer alone for about 45 minutes.

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge` or any
   write command on git; you must NOT mention any AI assistant, model, vendor or URL anywhere in
   the repository; document everything in English in `docs/`).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. The last entries of `docs/PROJECT_LOG.md` (the two most recent M5c entries), `docs/ISSUES.md`
   items I-012, I-033, I-034, I-036, I-039, and `benchmark/results/20261007-201137-full-owner-reps1/NOTE.md`.

## Context
The first full benchmark (`benchmark/results/20261007-201137-full-owner-reps1`) was executed while
the machine was busy. Its accuracy figures are expected to be valid (engines are deterministic) but
its speed, memory and busy-core figures are unreliable. The task is to re-run it on a quiet
machine, prove or disprove determinism, and replace the speed figures.

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (look for `bench.exe`,
   `whisper-cli.exe`: if one is running, do NOT start another benchmark; read its log instead).
2. **Build before measuring**: `cd src-tauri && cargo build --release --example bench`. Do not
   compile anything else once the benchmark has started.
3. **Quiet-machine check**: `./target/release/examples/bench.exe preflight` (exit code 0 = quiet,
   2 = not quiet). If it fails, diagnose without stopping the owner's processes: list the busiest
   processes, check `wsl -l -v` and whether Docker Desktop/`vmmemWSL` are running, then tell the
   owner exactly what to close (for example quit Docker Desktop, then `wsl --shutdown`) and
   re-run the preflight until it passes. Never use `--force` and never kill the owner's processes.
4. **Launch the benchmark in the background** with the same settings as the first run (all installed
   models, both whisper.cpp decodings, 95 samples, 1 repetition), from `src-tauri`:
   `./target/release/examples/bench.exe run --reps 1 --label full-owner-reps1-clean > ../tmp/bench_clean.log 2>&1`
   (create `tmp/` if needed; it is git-ignored). Expect about 45 minutes. While it runs, do
   NOTHING CPU-heavy (no builds, no tests, no screenshots, no UI automation). Read the log
   lightly (a few times, not in a tight loop). If the session is interrupted, the benchmark process
   keeps running: check `tasklist` and the log before doing anything else. Never start a second run.
5. **When it finishes** (`results written to ...` in the log):
   - confirm in `config.json` that `forcedDespitePreflight` is false and note `preflight.idleCpuPercent`
     and `system.json` power fields;
   - `python scripts/compare_runs.py benchmark/results/20261007-201137-full-owner-reps1 benchmark/results/<new folder>`
     (determinism of transcripts, then speed old versus new);
   - `python scripts/bootstrap_ci.py benchmark/results/<new folder>` and again with
     `--exclude en-it-05-owner` (that sample is suspect, see I-033);
   - read the new `summary.md`.
6. **If any transcript differs between the two runs**: do not hide it. Investigate (thread
   non-determinism, whisper.cpp, model load order), report it with examples, and state what is and
   is not affected. If everything is identical, say so: it proves accuracy does not depend on load.
7. **Document** (all English): append a dated entry to `docs/PROJECT_LOG.md` (Done / Verified /
   Failed or surprises / Not verified / Next) with the determinism result and a NEW speed table
   (RTF median, inference median and p95, cold load, peak memory, busy cores per configuration,
   measured on the quiet machine) clearly labelled as replacing the unreliable figures; update
   I-039 (resolved or not), I-012 (still open: only 1 repetition), I-036 and I-037 if the numbers
   changed the picture; update provisional decision D-032 if the speed picture changed the
   shortlist; update `README.md` and sections 3 and 7 of `docs/HANDOFF.md`; add a line to the old
   run's `NOTE.md` pointing to the clean run. Do not delete or rewrite the first run.
8. **Identity and privacy check**: search the repository for AI or vendor names (none allowed) and
   make sure no private content was written.
9. **Final message to the owner, in French**: what was run, the quiet-machine conditions, whether
   the transcripts were identical, the new speed table compared with the old one, the limits (one
   speaker, 1 repetition, CPU only), and the exact git commands for the owner to run (never run
   them yourself), checking `git status` first so the list of paths is complete. Propose the next
   step from the backlog in `docs/HANDOFF.md` (the timing study with 3 repetitions is next).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software without asking. Do not touch the owner's other app
  (port 1420) or their Docker/WSL work.
- Private audio (`wav/`, `benchmark/audio/`, `benchmark/samples-private/`) must never be quoted.
