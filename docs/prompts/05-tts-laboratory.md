# Prompt 05 — M6: text-to-speech laboratory

STATUS (2026-10-08): DONE except the owner's listening notes (D-038, project log entry "M6"). Kept for
reference; to record the notes, open a short session with the log entry as context.

Paste everything below the line into a new chat session. Prerequisites: the T5 import-tool work
(D-037) is committed, and no benchmark is running. This task does NOT need the owner's accent clips
(backlog item 6 stays open and can be done before or after). It needs the owner's approval before any
model download, and the owner's ears for the quality judgement.

---

You are continuing the project in this repository (AssistantCabinetAI-SpeechLab). The owner speaks
French: answer the owner in French in the chat; write every file, comment and commit message in
English. Work autonomously, but follow the project rules to the letter.

## Read first, in this order
1. `AGENTS.md` (mandatory rules: you must NOT run `git commit`, `git push`, `git merge` or any
   write command on git; you must NOT mention any AI assistant, model, vendor or URL anywhere in
   the repository; document everything in English in `docs/`; do not install software or change the
   system without the owner's approval).
2. `docs/HANDOFF.md` (state of the project, traps, commands).
3. The last entries of `docs/PROJECT_LOG.md`, `docs/DECISIONS.md` D-004, D-012 (espeak-ng GPL-3.0
   exposure) and D-037, `docs/ISSUES.md` (I-003, I-004), `docs/M0_FEASIBILITY.md` section 5 (candidate
   TTS models with licences), `Plan.md` section "C. Text-to-Speech Laboratory", and
   `docs/prompts/next-tasks.md` section T6.

## Context
M0 to M5 and the studies T2 to T4 are done for speech-to-text. Nothing exists yet for text-to-speech:
the contract `TextToSpeechProvider` (`voices`, `synthesize`, with `SynthesizeRequest` /
`SynthesizeResult` / `VoiceInfo` in `src-tauri/src/speech/types.rs` and `provider.rs`) is declared but
has no implementation. Goal of M6: find out, with measurements and the owner's listening notes, whether
sherpa-onnx can give usable French and English voices for the product, at what speed, under which
licence. Constraints from the plan: free and open-source first; reject non-commercial (CC-BY-NC) and
copyleft-incompatible (AGPL, GPL) voices without the owner's explicit approval (known: `vits-mms-fra`
is CC-BY-NC, Piper `fr_FR-tom` is AGPL, the espeak-ng phonemizer used by the Piper/Kokoro path is
GPL-3.0: document that exposure, do not hide it); everything offline at synthesis time; never
auto-play audio; gender is shown only when the voice's model card documents it; naturalness is an
opinion, record it as the owner's listening note, never as a measurement.

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe` or
   `whisper-cli.exe` running). Read `src-tauri/src/speech/types.rs`, `provider.rs`, `sherpa.rs`,
   `models.rs`, `download.rs`, `src-tauri/models-manifest.json`, the registry and the way the UI
   lists engines, so the TTS work follows the same patterns (manifest data, no UI hard-coding of one
   engine).
2. **Choose candidates and ask before downloading**: from `docs/M0_FEASIBILITY.md` section 5, propose
   a short list (for example one or two French voices and one or two English voices, plus a multi-voice
   model such as Kokoro if its French/English voices and licence are confirmed on the model card).
   For each: licence read from the official card (VERIFIED) or "unknown" (NOT VERIFIED), size, whether
   the espeak-ng phonemizer is involved. Show the owner the list with sizes (in French) and WAIT for
   approval before any download. Models go through the existing manifest and downloader (checksums
   pinned, D-015). If the sherpa-onnx crate version in use lacks the TTS API, say so and propose the
   smallest change; do not upgrade silently.
3. **Backend**: implement `TextToSpeechProvider` for sherpa-onnx (voice list with id, language,
   licence, gender only if documented, sample rate, speaker count; synthesis with a `speed` parameter;
   cancellation between sentences if the library cannot stop mid-generation: say which). Return the
   audio as 16-bit WAV bytes or a file in the app data folder, plus `generationMs` and `audioMs` so the
   real-time factor can be shown. Unit-test on short text; keep the old STT tests green
   (`cargo test --lib`, 123 tests at the time of writing).
4. **UI**: a TTS tab following the engine-agnostic pattern: text box, language and voice picker (with
   licence and a visible warning for restricted licences), speed control (only enabled when the voice
   supports it: test it, do not assume), a "Generate" button, then a SEPARATE "Play" button (never
   auto-play), pause/resume/stop where the platform audio element supports it, "Save WAV", generation
   time versus audio duration. Test it by driving the page over the DevTools protocol as described in
   `docs/HANDOFF.md` section 5 (port 1430, never touch port 1420).
5. **Measure** on a calm machine (`bench preflight` first, never `--force`): for each voice, 3 to 5
   fixed sentences of different lengths per language (take them from the public dataset scripts, which
   contain no private content), 3 repetitions: generation time, audio duration, RTF, memory if the
   existing probe can do it, cold start. Quote the preflight load with every speed figure and remember
   that between-run offsets of 20 % or more were seen (I-042): compare inside one run only.
6. **Listening notes**: generate the same sentences with each voice, save the WAV files under
   `benchmark/tts-samples/` ONLY if their licence allows it, otherwise keep them git-ignored; ask the
   owner (in French) to listen and give notes per voice (intelligibility, naturalness, accent, pronunciation
   of drug names and numbers, speed control). Record them verbatim as OPINION in the log. Do not rate
   voices yourself.
7. **Optional, only if the owner agrees**: use the TTS output as synthetic, licence-clean test audio
   for the STT benchmark (round trip: text -> voice -> STT), recording its provenance (voice, licence,
   `source.kind` = `synthetic`). Never mix these figures with the human-speaker figures.
8. **Document** (all English, nothing invented): `docs/PROJECT_LOG.md` (Done / Verified / Failed or
   surprises / Not verified / Next), `docs/DECISIONS.md` (provider design, chosen candidates, licence
   stance on espeak-ng), `docs/ISSUES.md`, a licensing table update, `README.md` (how to run the TTS
   tab), sections 3 and 7 of `docs/HANDOFF.md`, `docs/prompts/README.md`. Mark every claim VERIFIED
   (with evidence) or NOT VERIFIED (with the steps).
9. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the
   process list recorded by `bench preflight` can contain an editor name, replace it by a neutral label
   in the result folder's `config.json`), make sure no private content was written, and that `git
   status` shows no model file or large audio file to add by mistake.
10. **Final message to the owner, in French**: what was done, verified, not verified, limits, the exact
    git commands for the owner to run (never run them yourself; check `git status` first so no path is
    forgotten; one English sentence per commit message, imperative, about 72 characters, no trailer),
    and prepare the prompt file for the following step from the backlog in `docs/HANDOFF.md`
    (`docs/prompts/06-...`: M7 Windows/macOS packaging validation, or the private accent clips if the
    owner has provided them by then).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers. Naturalness is opinion.
- Do not commit. Do not install software or download anything without asking. Do not touch the
  owner's other app (port 1420) or their Docker/WSL work.
- Very long inline shell or Python scripts with apostrophes were rejected by the agent shell: write
  script files and run them.
- Never run two benchmarks at once; do not compile, test or record while one runs.
