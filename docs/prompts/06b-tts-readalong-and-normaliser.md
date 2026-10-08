# Prompt 06b — M6c: read-along highlight, text normaliser, kristin, better free French voice

Paste everything below the line into a new chat session. Prerequisites: the M6/M6b/M6c work (D-038 to
D-041) is committed and no benchmark is running. The owner has already taken the decisions this task
needs (see below), so work autonomously. Ask the owner (in French) before any download or any installation
of a Python package, and for the listening notes at the end.

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
3. The last entries of `docs/PROJECT_LOG.md` (M6, M6b, M6c), `docs/DECISIONS.md` D-038, D-039, D-040
   (and its 2026-10-08 update), D-041, `docs/TTS_LICENSES.md`, `docs/TTS_LISTENING_NOTES.md`,
   `docs/ISSUES.md` (I-048 to I-057).

## Decisions already taken by the owner (do not reopen them)
- **Highlight**: the SENTENCE being read gets a grey background (must work in light and dark themes);
  long texts SCROLL AUTOMATICALLY so that the sentence being read stays visible. NOT wanted in this
  release: click on a sentence to jump to it (maybe later, for an add-on or editorial use with a lot of
  text; keep sentence segments identified so it can be added), paragraph shading, word-level highlight.
- **French voice**: the digits-to-words normaliser is wanted because the Coqui css10 voice (the only
  freely usable French voice found) does not speak digits. Public-domain material is to be preferred.
- **Licence stance**: a beta is acceptable only if it needs no paid licence. Offer by default only voices
  rated `clear` or `attribution`; never ship or recommend `review` or `excluded` voices (Piper siwis,
  Kokoro, Piper gilles stay installed as quality references). No legal consultation for now.
- **Kokoro** is deprioritised (hiss heard on every Kokoro voice, slower than real time, rated `review`).
- The accent clips were not found: do not work on T5.

## Context
The TTS tab (`src/components/TtsPanel.tsx`) generates one WAV per request and plays it in a separate player
(never auto-play, a project rule). The provider (`src-tauri/src/speech/tts.rs`) synthesises the whole text in
one library call and reports one duration; the library gives samples only, no word or sentence timings,
so sentence timings must be MEASURED by synthesising sentence by sentence. Owner's blind listening
(`docs/TTS_LISTENING_NOTES.md`): English `clear`/`attribution` voices are excellent; the free French voice
is weak (clarity 3, accent 2, digits missing). Synthesis is not deterministic (I-056).

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe`,
   `whisper-cli.exe`, `tts.exe` running; if a `speechlab.exe` window of the owner is open, do not touch
   it, and ask the owner to close it before any timing measurement), `bench preflight` before measuring.
2. **Sentence splitter** (new Rust module, unit-tested): French and English; handles abbreviations
   (M., Mme, Dr, Pr, St, etc., env., n°, vs., e.g.), decimals (12,5 and 12.5), times (14 h 30, 14:30,
   9h00), ellipses, guillemets and quotes, line breaks and blank lines (a paragraph break is a longer pause).
   Return the original text span of each sentence (start and end offsets) so the UI can highlight it.
3. **Text normaliser** (new Rust module, unit-tested, French and English): cardinals (French 70, 80, 90
   as "soixante-dix", "quatre-vingts" with the correct plural and "quatre-vingt-un"), ordinals (1er, 2e, 12th),
   decimals, percentages, currency, times, dates ("12 mars 2026", "March 12th"), units (mg, g, ml, kg,
   °C, h), common abbreviations, and phone numbers or identifiers (digit by digit or in pairs, say which).
   Use the owner's domains for test sentences (medical, administrative, IT: "500 mg trois fois par jour",
   "10.30", "9h00", "facture numéro 2045"; see I-034 for the time formats). It is applied PER SENTENCE and
   only to the text sent to the voice, never to the displayed text, and enabled by a manifest flag per voice
   package (on for the Coqui css10 voice; off for phonemizer voices, which already speak digits), with a UI
   toggle that shows the normalised text for transparency. Verify on the real Coqui voice that digits are now
   spoken (recogniser round trip as a rough check, AND the owner's ear at the end).
4. **Per-sentence synthesis with timings**: extend the provider so a request is split into sentences, each
   normalised if needed and synthesised separately, then joined into ONE WAV with a short silence (about
   250 ms after a sentence, longer at a paragraph break; choose and record the values). `SynthesizeResult`
   gains `segments` (text span, start ms, end ms, in the original text). Cancellation works between
   sentences and inside one (callback). A one-sentence request behaves as today. Check that the extra
   per-call overhead is small (measure RTF before and after on the same voices, in one run).
5. **Read-along view in the UI**: after Generate, show the original text as sentence spans in a read-only
   view (the textarea cannot style parts of its text; editing returns to the textarea and invalidates the
   highlight). Drive the highlight from the audio element's `currentTime` (use `requestAnimationFrame`, not
   the 4 Hz `timeupdate`): the active sentence gets a grey background with enough contrast in both themes
   and `aria-current`. Auto-scroll smoothly to keep the active sentence in view; if the user scrolls by hand,
   suspend auto-scroll for a few seconds and offer a "Follow reading" toggle. Pause keeps the highlight, Stop
   clears it, seeking with the player's own bar moves it. The pure function "time -> sentence index" is
   unit-tested with vitest. Playback still starts only when the user presses play.
6. **UI test over the DevTools protocol** (port 1430 only, `docs/HANDOFF.md` section 5; never touch port
   1420 or the owner's processes): generate a text of 6 sentences with a Piper voice, set
   `audio.currentTime` to the middle of each segment and check which span is highlighted; generate a text
   of 40 sentences and check that the container scrolls when the highlight moves down; check the dark theme
   contrast by reading the computed colours; clean up what the test creates.
7. **`kristin` (English, public domain, already installed)**: in one calm run (preflight first, never
   `--force`) measure it together with the other `clear`/`attribution` voices (ljspeech, libritts_r, Coqui)
   so the comparison is inside one run (I-042). Add it to `scripts/tts_listening.py` and prepare a NEW blind
   listening page with kristin, ljspeech and libritts_r (so the owner can pick the best public-domain English
   voice) and Coqui with the normaliser ON versus OFF on a date sentence. Give the owner the exact steps in
   French (open the page, headphones, fill the form, copy the notes, paste in the chat) and record the notes
   verbatim in `docs/TTS_LISTENING_NOTES.md` as opinion.
8. **Better free French voice (research first, no download without approval)**: read cards and release lists
   (read-only) for freely usable French TTS voices that sherpa-onnx can run, following `docs/TTS_LICENSES.md`
   (follow the "fine-tuned from" chain). Include Chatterbox (MIT; multilingual; the community `Chatterbox-TTS-French` fine-tune, CC-BY-4.0) and Qwen3-TTS (Apache-2.0, 0.6B and 1.7B), which are NOT sherpa-onnx models (another runtime, voice cloning from a reference clip, large): read their cards for training-data licences and watermark or cloning terms, check whether any runs on this CPU without a GPU (ONNX or GGUF builds exist) and estimate cost, but download nothing without approval. Report candidates with licence, lineage, phonemizer and size. Then
   propose to the owner, in French, the conversion of the Piper `fr_FR-mls-medium` checkpoint (CC BY 4.0,
   trained from scratch; not in the sherpa-onnx release): what tool, which packages to install (preferably
   in an isolated virtual environment inside `vendor/`), disk and time cost, and what is NOT verified. Do it
   only after approval, then measure and add it to the next listening page.
9. **Document** (all English): `docs/PROJECT_LOG.md` (Done / Verified / Failed or surprises / Not verified /
   Next), `docs/DECISIONS.md` (splitter and normaliser design, segment format, silences), `docs/ISSUES.md`
   (close or update I-055; open new ones), `docs/TTS_LICENSES.md` (new voices), `docs/TTS_LISTENING_NOTES.md`,
   `README.md`, sections 3 and 7 of `docs/HANDOFF.md`, `docs/prompts/README.md`. Mark every claim VERIFIED
   (with evidence) or NOT VERIFIED (with the steps).
10. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the process
    list recorded by `bench preflight` can contain an editor name, replace it by a neutral label in the result
    folder's `config.json`), no private content, and `git status` must show no model file or audio file.
11. **Final message to the owner, in French**: what was done, verified, not verified, limits, what the owner
    must listen to or test (concrete steps, examples), the exact git commands for the owner to run (never run
    them yourself; check `git status` first so no path is forgotten; one English sentence per commit message,
    imperative, about 72 characters, no trailer), and the next prompt file to use:
    `docs/prompts/06-packaging-validation.md` (M7), unless the owner has asked for something else.

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers. Naturalness is the owner's
  opinion. Synthesis is not deterministic: repeat before concluding from one generation.
- Do not commit. Do not install software or download anything without asking. Do not auto-play audio.
- Very long inline shell or Python scripts with apostrophes were rejected by the agent shell: write script
  files and run them. `python -I` ignores `PYTHONIOENCODING`: use `python -I -X utf8`.
- Never run two benchmarks at once; do not compile, test or record while one runs. A running
  `pnpm tauri dev` recompiles when Rust files change: tell the owner to restart it if the app looks stale.
