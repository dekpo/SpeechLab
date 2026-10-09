# SP-6 launcher: Speech 6, read aloud

**Tab name: "Speech 6 — Read aloud".** Working folder: `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`.
Branch: `feat/speech-read-aloud`, from `kb/integration`.

Drag into the tab: `00-MASTER.md`, this file, `docs/SPEECH.md`, `docs/CHAT-UX-ASSESSMENT.md` (section 6), SpeechLab `docs/SPEECH_ENGINE_EVALUATION.md` (section 5) and `docs/TTS_LICENSES.md`.

**Written before the earlier phases ran: re-check every fact against the repository first.**

## Goal

The owner's draft step 5, first half: a button that speaks an answer, with local voices only, off by default, with names and
numbers spoken correctly.

## Preconditions (stop if not met)

SP-4 accepted. The revised voice rule (read-aloud of text already on screen, local voices only) is recorded in `docs/DECISIONS.md` (SP-1). Q-06 answered (path order); Q-01/espeak-ng questions answered **if** the Piper path is in scope. The KB lots 3 and 9 are merged if spoken forms are in scope (otherwise skip task 4 and say so).

## Tasks

1. **Spike first (small)**: system voices through `window.speechSynthesis` with `voice.localService === true` only; wait for `voiceschanged`; a machine code and a plain message when no local voice matches the locale (refuse, never fall back to an online voice). Report which French and English local voices exist on the owner's Windows machine. Then a **listening comparison** by the owner: the best local French voice against SpeechLab's recorded Piper samples (the owner generates them with SpeechLab's `tts_listening.py` page, or you ask for the sentences used in `benchmark/tts/listening`). Record the owner's scores verbatim as opinion, in the style of SpeechLab's `TTS_LISTENING_NOTES.md`. This decides the path; do not decide it yourself.
2. **Speakable text** (`speech/speakable.rs`, data in `resources/speech/rules/{fr-FR,en-US}.json`): derive the text to speak from the answer's Markdown **source**: remove links, citation marks, code and table syntax (tables become sentences by a rule you document), apply the SpeechLab normaliser's behaviour as data (cardinals, ordinals, decimals, thousands, percentages, currency, times, dates, units with the plural rule of each language, abbreviations, phone numbers). Port the algorithm, not the file: the source guard forbids French words and accents in Rust, so every word lives in the JSON packs. Turn SpeechLab's unit cases (`normalise.rs` tests, 15 of them) into fixtures under `src-tauri/tests/`. The model never produces the spoken text (SI8).
3. **Port and adapters**: `TextToSpeechProvider`; the system-voice adapter lives in the front end behind the same port shape; add a Piper TTS **sidecar** (a separate executable with its own licence notice) only if the owner accepted the conditions; never link sherpa-onnx into the main executable.
4. **Spoken forms**: lookup order manual `spoken_form` attribute, then pack `title_spoken` and rules, then the engine; respell the text sent to the engine, never the displayed text.
5. **UI**: a `ReadAloudButton` in the answer action row beside copy, edit, regenerate; stop; a speaking indicator; speed and voice in the Speech settings block. **Nothing plays by itself.** "Read answers automatically" is not built here. No sentence highlight in v1 (answers are rendered as Markdown; mapping spans onto the rendered tree is a separate problem).
6. Privacy text: a spoken answer can be overheard; the setting's description says so.

## Tests and acceptance

SI7 (`a_non_local_voice_is_never_selected`), SI8, speakable-text fixtures (names, dates, doses, percentages, a table, a citation), a no-local-voice case, SI9, catalogue parity, source guard.
Human test: an answer with a name, a date, a dose and a citation is spoken without citation marks; stop works; no local voice gives a clear message; the owner compares the voice with the listening result.

## Final message (French)

Done, verified, not verified (voice quality is opinion), limits, git block, then: "Prochain onglet: Speech 8 — Prove it (`SP-8-prove-it.md`), ou Speech 7 — Spoken conversation (`SP-7-conversation-mode.md`) si vous décidez de la construire."
