# Prompts for new sessions

How to continue the project in a fresh chat tab without losing context:

1. Open a new tab/session in the editor, with this repository as the working folder.
2. Paste the content of one prompt file (below the horizontal line for `01-...`; preamble P0 plus
   one task block for `next-tasks.md`).
3. The agent reads `AGENTS.md`, `docs/HANDOFF.md` and the project log, then works. At the end it
   gives you git commands: YOU run them, the agent never commits.
4. If a session is interrupted, start a new one with the same prompt: the agent checks the state
   (git, running processes, logs) before acting. Long benchmark processes keep running by
   themselves.

| File | Use it for |
|---|---|
| `01-clean-benchmark-rerun.md` | done (kept for reference): re-run the full benchmark on a quiet machine |
| `02-long-audio-vad-chunking.md` | done (kept for reference): cut long audio at silences (VAD) and compare with whole-clip decoding |
| `03-drug-name-handling.md` | the immediate next task: drug names and key terms (hotwords, prompts, dictionary correction) |
| `next-tasks.md` | private accent clips, TTS, packaging, final report (the timing study T2, the re-run and the chunking T3 are done; T4 drug names now has its own file) |

Two sessions must never work on the same files at the same time, and never two benchmarks at once:
tell each session which branch you are on, and finish or commit one task before starting the next.
