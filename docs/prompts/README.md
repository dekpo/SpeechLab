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
| `03-drug-name-handling.md` | done (kept for reference): drug names and key terms (hotwords, prompts, dictionary correction) |
| `05-tts-laboratory.md` | done (kept for reference): M6 text-to-speech laboratory |
| `06b-tts-readalong-and-normaliser.md` | done (kept for reference): sentence highlight with auto-scroll, digits-to-words normaliser, `kristin` measurement and listening, search for a better free French voice (D-040, D-041, D-042) |
| `06-packaging-validation.md` | done (kept for reference): M7 Windows packaging validation (D-044, D-045); the macOS steps are in `docs/MACOS_VALIDATION.md`, NOT VERIFIED |
| `07-final-report.md` | done (kept for reference): M8 final report, licensing table and recommendation (`docs/SPEECH_ENGINE_EVALUATION.md`, D-047); the next steps are the owner's choices listed in `docs/HANDOFF.md` section 7 (items 11 to 14) |
| `next-tasks.md` | TTS, packaging, final report (the timing study T2, the re-run and the chunking T3 are done; T4 drug names is done; the accent task T5 was dropped by the owner, D-046) |

Two sessions must never work on the same files at the same time, and never two benchmarks at once:
tell each session which branch you are on, and finish or commit one task before starting the next.
