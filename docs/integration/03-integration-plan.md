# Integration plan: speech-to-text and text-to-speech in AssistantCabinetAI

Status: **PROPOSAL for the owner's review** (2026-10-09). Becomes the plan when the owner accepts it (D-049, proposed).
Inputs: `01-owner-draft-and-review.md` (the owner's draft and its review), `02-needs-and-structure-analysis.md` (facts F1 to
F20, reuse map, architecture, invariants SI1 to SI12), `docs/SPEECH_ENGINE_EVALUATION.md` (evidence), D-048 (SpeechLab frozen,
macOS on hold).

How to use: the owner submits **one phase at a time**, each in its own chat tab, by dragging the launcher named in the phase
(`docs/prompts/integration/`) plus the master file. A phase ends with a report, a proposed human test, an updated status file
and the git commands for the owner. The next phase starts only when the owner says the previous one is accepted.

---

## 1. Overview

```text
SpeechLab (frozen)                         AssistantCabinetAI (live: kb/integration)
  evidence, bench tool, reuse map   ──►     SP-1 contracts ► SP-2 port+sidecar ► SP-3 models+settings ► SP-4 dictation
  SP-0: freeze, decisions, practice-PC probe                                              │
                                                                                          ▼
                          KB lots 2, 3, 6, 9 merged ──────────────►  SP-5 KB-assisted spelling (repair-first)
                                                                                          │
                                                          SP-6 read aloud ► SP-7 conversation mode (optional) ► SP-8 prove it
macOS: ON HOLD (waiver Q-04). Code stays cfg-free and compiles in CI; nothing is validated on a Mac.
```

| Phase | Tab name | Repo | Kind | Size | Depends on | Can run beside the KB programme? |
|---|---|---|---|---|---|---|
| SP-0 | Speech 0 — Freeze and practice-PC probe | SpeechLab | tooling + decisions | S | — | yes (no AssistantCabinetAI file touched) |
| SP-1 | Speech 1 — Contracts and decisions | AssistantCabinetAI | documents only | M | SP-0 decisions Q-01 to Q-05 | yes (docs; avoid files KB lots edit the same day) |
| SP-2 | Speech 2 — Port and whisper sidecar | AssistantCabinetAI | backend | L | SP-1 | yes (new files only) |
| SP-3 | Speech 3 — Models and settings | AssistantCabinetAI | backend + settings UI | M | SP-2 | partly (shared settings files) |
| SP-4 | Speech 4 — Dictation in the chat | AssistantCabinetAI | frontend + CSP | L | SP-3 | sequence after KB lot 7 or 8 for `Composer`/locale merges |
| SP-5 | Speech 5 — Names and spelling | AssistantCabinetAI | KB integration | M | SP-4 and KB lots 2, 3, 6, 9 merged | after those lots |
| SP-6 | Speech 6 — Read aloud | AssistantCabinetAI | backend + UI | L | SP-4, Q-06 | after SP-4 |
| SP-7 | Speech 7 — Spoken conversation (optional) | AssistantCabinetAI | design + frontend | M | SP-4, SP-6, Q-07 | owner's call |
| SP-8 | Speech 8 — Prove it | AssistantCabinetAI | validation, packaging | M | all chosen phases | last |

Size guidance as in the KB plan: S up to 1 day, M about 2 to 3 days, L about 4 to 6 days of agent-plus-review time.
Guidance, not a promise.

---

## 2. Decisions the owner must take (with my recommendation)

Nothing here may be decided by an agent. SP-0 puts these in front of the owner; SP-1 records the answers in
AssistantCabinetAI's `docs/DECISIONS.md`.

| # | Question | Options | Recommendation | Needed by |
|---|---|---|---|---|
| Q-01 | May a **CC BY 4.0** speech model (Parakeet, Canary) be used? KB D7 allows CC BY "only by an explicit later decision" | no (Whisper MIT only); yes for Parakeet with attribution | Decide **after the practice-PC probe**. Until then Whisper (MIT) is the only engine in scope. If the probe shows no Whisper size is fast and accurate enough on the 2019 PC, Parakeet becomes the case for a yes (also needs 1.6 to 1.9 GB) | SP-2 start for the first engine; final at SP-3 |
| Q-02 | Process model | engine linked in the main executable; engine in a separate sidecar | **Sidecar only.** Keeps GPL code, 1.9 GB and crashes out of the product, gives real cancellation | SP-1 |
| Q-03 | Base branch for speech work | `main`; `kb/integration` | `kb/integration` (the KB programme keeps `main` untouched until lot 11). Speech branches `feat/speech-*` start from it and merge back by pull request | SP-1 |
| Q-04 | macOS hold versus "Windows and macOS both mandatory" | keep the rule and block; waive for speech while no Mac exists | **Recorded waiver**: speech code must compile and unit-test on both (CI unchanged), contain no `cfg`, and every macOS validation line stays NOT VERIFIED and on hold; no macOS claim in any release note. Revisit when a Mac is available | SP-1 |
| Q-05 | How speech model files reach a practice machine (no mandatory Internet) | installer variant that carries the model; owner side-loads a folder; the practice server (Mac mini) serves model files over the LAN (files only, never audio) | Dev: explicit download with checksum. Pilot: **side-load or an installer variant**; defer the LAN mirror (it adds server work and a network path next to speech) | SP-3 |
| Q-06 | Read-aloud path order | system voices first; Piper sidecar first | **System voices first** (local-only guard), then a listening comparison with Piper; Piper only with the licence conditions accepted. French interim = the best local system voice or Coqui | SP-6 start |
| Q-07 | Spoken conversation mode | build; do not build | Decide after SP-6 is in use. If built: push-to-talk first, confirmation before send, no path to file actions, echo control | SP-7 |
| Q-08 | Acceptance budgets | owner proposes; see section 4 | Fix after the probe | SP-3 |
| Q-09 | Licence of AssistantCabinetAI code and the credits view for CC BY / BSD / MIT / GPL notices | — | Needed before any distribution; not blocking the early phases | SP-8 |

---

## 3. Engine decision (the probe decides, then the owner)

```text
Practice-PC probe (SP-0): for whisper.cpp base, small (and large-v3-turbo if the owner approves the 574 MB download) and,
if Q-01 is open, Parakeet: RTF on 10 s of speech, peak memory, busy cores, background load, accuracy on the owner's reference
recordings (WER with the number of sentences).
 ├─ base or small within budget (Q-08) ──► first engine = whisper.cpp sidecar (MIT, existing pattern, KB D7). Parakeet stays an option.
 ├─ no Whisper size within budget, Parakeet within budget ──► owner decides Q-01 (CC BY 4.0) and the memory; then add the worker adapter.
 ├─ nothing within budget ──► dictation is deferred; typed chat stays; options: faster PC, smaller expectations (base + KB repair),
 │                            or reopening D7 (speech on the Mac mini; audio crosses the LAN) which is the owner's rule to change, not ours.
 └─ the probe could not be run ──► build the port and the fake provider anyway (SP-2 is engine-independent), do not ship dictation.
```

SpeechLab's evidence for the table's starting point: whisper.cpp base RTF 0.29 to 0.34 at 8.4 % WER; small RTF 1.1 to 1.3 at
3.9 %; Parakeet RTF about 0.10 at 2.3 % with 1.6 to 1.9 GB (all on a 2024 laptop). With KB repair the base model's
name errors may become acceptable, but that is **NOT VERIFIED** and is exactly what SP-5 measures.

---

## 4. Proposed budgets and acceptance (to confirm as Q-08)

| Item | Proposed target (practice PC) | Basis |
|---|---|---|
| Dictation, 10 s utterance | text back within about 10 s (RTF at most 1.0); the interface never freezes | whisper small misses this on the 2024 laptop; base and Parakeet meet it |
| Extra resident memory while transcribing | at most about 1 GB unless the owner accepts more | Parakeet 1.6 to 1.9 GB, whisper small 0.4 to 0.5 GB |
| Model on disk | whisper base 60 MB, small 190 MB; Parakeet 487 MB | manifest |
| Start-up | speech adds no measurable delay to application launch (sidecar started on first use) | SpeechLab: window visible about 130 ms; table 750 ms |
| Accuracy | reported as WER with interval and the number of speakers; no threshold promised until the real speaker is measured | single-speaker evidence only |
| Privacy | SI1 to SI12 green; offline proof with the speech sidecar and application blocked from the network | SpeechLab M7 procedure |

---

## 5. Phases

Every phase: work on a branch from `kb/integration` (Q-03), keep `cargo test`, `pnpm run test`, `pnpm run build` green,
write the lot report, propose a human test, update the status file, give the owner the git block (branch, `git add` of explicit
paths, one Conventional Commit line of at most 72 characters, no body, no AI attribution). The status file is
`docs/SESSION-SPEECH-STATUS.md` in AssistantCabinetAI (local, like the KB one), created in SP-1.

### SP-0 — Freeze SpeechLab, take the decisions, probe the practice PC

- **Repo**: SpeechLab only. No AssistantCabinetAI file is touched.
- **Goal**: close SpeechLab cleanly, put the decisions Q-01 to Q-09 in front of the owner, and measure the real machine.
- **Preconditions**: M8 report committed by the owner.
- **Work**
  1. Owner-run: tag the final SpeechLab state (`git tag speechlab-m8-final`, push the tag) once the M8 and integration documents are committed. Agents never tag.
  2. Add `scripts/practice_pc_probe.ps1` and `docs/integration/PRACTICE_PC_PROBE.md`: a self-contained, read-only procedure for the practice PC (collect CPU model, cores, RAM, OS, power plan; `bench preflight`; run `bench` on a small fixed subset with the candidate models; write an aggregate-only result file). It uses the existing `bench` binary and the existing dataset audio of the owner's own voice; the results are numbers only.
  3. Write `docs/integration/PROBE_RESULTS.md` (empty template + how to fill it); the owner (or the GP's PC session) fills it.
  4. Export `heard -> expected` pairs from the **public** result folders (clean run, T4 runs) as a candidate fixture for the KB `repair-fr.json` contract: key terms and names only (drug names, IT terms, "Lausanne"-style place names), with the engine that produced each. Aggregates and the public script sentences only; nothing from private folders.
  5. A one-page **decision sheet** for Q-01 to Q-09 with the evidence for each (extract from this plan).
- **Acceptance**: the probe script runs on the SpeechLab laptop and writes the template (VERIFIED by running it here); the owner has the sheet; D-048 is committed.
- **Launcher**: `SP-0-freeze-and-probe.md`.

### SP-1 — Contracts and decisions in AssistantCabinetAI (documents only)

- **Goal**: the speech work is allowed, specified and bounded in the AssistantCabinetAI repository before any code, as KB lot 0 did for the KB.
- **Preconditions**: owner answers Q-02, Q-03, Q-04 at least; Q-01 may stay open.
- **Work**
  1. `docs/DECISIONS.md`: a section "Speech: workstation-local dictation and read-aloud (date)" recording Q-01 to Q-09 as decided or open, the invariants SI1 to SI12, the macOS waiver, the unused `/v1/audio/*` routes, and the revision of the voice rule for read-aloud of text already on screen (`CHAT-UX-ASSESSMENT.md` item 6 says this belongs in DECISIONS before the code).
  2. `docs/SPEECH.md`: the canonical spec (ports, flows, layout, settings, error codes, privacy table, packs, tests) from analysis sections 5 to 10.
  3. Update `docs/ARCHITECTURE.md` (ports), `docs/PRIVACY-AND-SECURITY.md` (audio and transcripts, temp files, hint leakage, read-aloud), `docs/LANGUAGE-AND-LOCALE.md` (speech locale mapping, speech packs as data), `docs/CLIENT.md` (mic button, read-aloud action), `docs/ROADMAP.md` (a speech track beside K-A to K-E), `models/LICENSES.md` (rows for each speech model considered, with the licence read from the exact card), `.cursor/rules/` (a `speech-programme.mdc` modelled on `kb-programme.mdc`).
  4. Create `docs/SESSION-SPEECH-STATUS.md` (local).
  5. Do **not** edit `.cursor/rules/v0-sprint.mdc` beyond what the decision requires, and do not touch KB files other than cross-references.
- **Tests**: none new; the existing suites stay green (docs only).
- **Acceptance**: the owner can read one spec and one decision table and say yes or no; no code changed.
- **Launcher**: `SP-1-contracts.md`.

### SP-2 — Speech-to-text port and the whisper.cpp sidecar (backend only)

- **Goal**: `SpeechToTextProvider`, a deterministic fake, the whisper.cpp sidecar adapter, the temp-audio lifecycle and the Tauri commands, with no user interface.
- **Preconditions**: SP-1 accepted; engine choice from the probe (Section 3) allows whisper.cpp, or the owner says to build the port anyway.
- **Work**
  1. `src-tauri/src/speech/` as in analysis section 5.2: `mod.rs`, `stt.rs`, `fake.rs`, `whisper_cli.rs`, `temp_audio.rs`, `manifest.rs`. ASCII only, no `cfg`, errors as machine codes in `error.rs` (`speech_engine_unavailable`, `speech_model_missing`, `speech_audio_too_short`, `speech_audio_too_long`, `speech_cancelled`, `speech_engine_failed`, `speech_locale_unsupported`).
  2. Commands `speech_status`, `speech_transcribe`, `speech_cancel` registered with minimal edits to `lib.rs`, `commands.rs` and `src/lib/ipc.ts`.
  3. Sidecar discovery by the existing `sidecar_search_roots`; the fetch script `scripts/fetch-speech-resources.ps1` (checksum-pinned, git-ignored target, same README precedent as Tesseract). **Do not declare `externalBin` in `tauri.conf.json`** (F5). Spike, and report, whether a platform overlay file (`tauri.windows.conf.json`) is merged on Windows only; if it works, that is the path for bundling (SP-8).
  4. Cancellation by dropping the future and killing the child (`cancellation.rs` pattern); one run at a time.
  5. Tests: contract tests on the fake; temp-audio removal on success, error, cancel; the startup sweep; SI1, SI2, SI6, SI9, SI10, SI11; an `#[ignore]` test with the real sidecar and a short synthetic clip.
- **Files**: new `speech/` module and tests; one line each in `lib.rs`, `error.rs`, `ipc.ts`, locale catalogues (error codes). 
- **Acceptance**: a command-line style test (or a Tauri command invoked from a test) transcribes a short clip on this Windows machine; cancelling kills the child and removes the temp file; the suites are green on both CI platforms.
- **Human test**: none visible (no UI); offer the printed output of the ignored test and a process-list check after cancel.
- **Launcher**: `SP-2-port-and-sidecar.md`.

### SP-3 — Models, availability and settings

- **Goal**: the application knows whether speech can work on this machine and lets the user control it, before any microphone button exists.
- **Work**
  1. `resources/speech/models-manifest.json` (id, display key, languages, size, SHA-256, licence, source, engine) and its verification; a model folder under the app-local data directory; a marker file after verification (SpeechLab design).
  2. Install path per Q-05: dev download with checksum (explicit user action, separate from inference); side-load of a folder; the manifest verifies either.
  3. Settings (`settings.rs`, named default constants, clamped): `speech_enabled` (false until a model verifies), `stt_model`, `stt_language_override` (None = follow the locale), `speech_hints` (off), later `tts_*`. A "Speech" block in the Advanced group of `SettingsDialog.tsx` with a status line (ready, model missing, engine missing) and the install/side-load action; strings in both catalogues.
  4. `models/LICENSES.md` rows verified from the exact card.
- **Tests**: manifest verification (good, corrupted, missing), settings round trip and clamping, availability matrix, catalogue parity; SI9.
- **Acceptance**: with no model the settings block says so and nothing else changes; with a verified model it says ready. Owner can switch speech off and the application behaves as before.
- **Launcher**: `SP-3-models-and-settings.md`.

### SP-4 — Dictation in the chat

- **Goal**: the owner's draft step 3, built to the invariants.
- **Preconditions**: SP-3 accepted; the CSP decision (static worklet file preferred) made in the phase.
- **Work**
  1. Port `src/audio/recorder.ts` / `wav.ts` / level analysis from SpeechLab; ship the worklet as a static file; extend the CSP minimally (`media-src` only if playback is needed here; none for dictation alone). Capture 16 kHz mono PCM, processing flags off, level and clipping warning.
  2. `MicButton` and `DictationStatus` components; one mount in `Composer`; a `useDictation` state machine (`unavailable | idle | requesting-permission | recording | transcribing | done | error`) with cancel at every state; insertion at the cursor; one-step undo; Enter and the Send button unchanged; focus returns to the textarea; `aria-live` status.
  3. Machine codes to localised sentences (permission denied, no microphone, too quiet, too short, too long, engine missing, cancelled, failed); the permission prompt behaviour documented (WebView2 per origin; macOS on hold).
  4. Timing line (counts and milliseconds only) behind the existing `write_timing_log` setting.
  5. Human test protocol with the pilot GP's voice and microphone on `fixtures/gp-sandbox/` content, fictional only.
- **Tests**: vitest for the state machine and insertion; SI3 (`dictation_never_calls_send`), SI4, SI5 (needs the KB present: snapshot unchanged), catalogue parity; manual: microphone prompt in the release origin; cancel at each stage; offline.
- **Acceptance**: press, speak, press, edited text in the box, explicit send; stop works; with the engine missing the button explains and chat is unaffected.
- **Launcher**: `SP-4-dictation.md`.

### SP-5 — Names and spelling from the Knowledge Base

- **Goal**: the owner's central requirement: better understanding and spelling of names, organisations and medicines through the KB.
- **Preconditions**: KB lots 2 (phonetics), 3 (ports, packs), 6 (selection mapping) and 9 (`surface`, `repair`) merged into `kb/integration`; SP-4 accepted.
- **Work**
  1. `SpeechHintSource` implemented over `knowledge::surface_forms(purpose = SpeechHints)` and `knowledge::repair::propose`; KB mode `off` gives an empty source; never a dependency of anything but this trait.
  2. **Repair-first**: after transcription, show `heard -> display` suggestion chips under or inside the composer; accept replaces the span; ignore leaves it; `Ask` shows the alternatives; never a silent replacement; the dictated text never changes the KB.
  3. Contract fixture: take the candidate pairs of SP-0 into `tests/fixtures/knowledge/repair-fr.json` **with the must-not-change pairs**; run the repair against them (this is where SpeechLab's measured errors become the KB's regression set).
  4. Optional hint list to the engine behind `speech_hints` (default off): short, selection-scoped, **not on a command line**; with `whisper-cli` the prompt is a command-line argument, so by default no hints go to it (analysis 8.3). Measure the effect with the SP-0 probe harness before keeping the option.
  5. Settings text explaining the privacy trade-off in plain French.
- **Tests**: repair fixtures (positive and must-not-change); selection-first (I7); `hints_never_leave_the_selection`; `kb_off_gives_no_hints`; SI5, SI6, SI12; vitest for chips.
- **Acceptance**: with a fictional KB (the KB sandbox) a dictated misspelled surname or drug name produces a suggestion that the user accepts; the KB row hash is unchanged afterwards.
- **Launcher**: `SP-5-names-and-spelling.md`.

### SP-6 — Read aloud

- **Goal**: the owner's draft step 5, first half: a button that speaks an answer, local only.
- **Preconditions**: SP-4 accepted; Q-06; the revised voice rule recorded in SP-1.
- **Work**
  1. **Spike (first, small)**: system voices through `speechSynthesis` with the `localService` filter, refusal with a machine code when no local voice matches the locale, `voiceschanged` handling; a listening comparison of the best local French voice against SpeechLab's Piper samples (the owner's ear is the judge; record it as opinion, as `TTS_LISTENING_NOTES.md` does).
  2. `speakable.rs` + `resources/speech/rules/{fr-FR,en-US}.json`: derive the speech text from the answer's Markdown source (strip links, citations, tables to a sentence form decided in the phase), apply the number/date/unit rules (SpeechLab normaliser logic, rewritten as data; its unit cases become fixtures), apply KB spoken forms (`spoken_form`, `title_spoken`) for names in the cited sources.
  3. `TextToSpeechProvider` port, the system-voice adapter (frontend) and, only if accepted, a Piper TTS sidecar adapter (separate executable, own licence notice).
  4. UI: a `ReadAloudButton` in the answer action row, stop, speaking indicator, speed from settings; **no automatic reading**; no highlight in v1.
- **Tests**: SI7, SI8, speakable-text fixtures, number/date/unit cases, a local-voice-only test, catalogue parity.
- **Acceptance**: an answer with a name, a date, a dose and a citation is spoken correctly and without the citation marks; stopping works; with no local voice the button explains.
- **Launcher**: `SP-6-read-aloud.md`.

### SP-7 — Spoken conversation (optional)

- **Goal**: the owner's draft step 5, second half, designed to the rules.
- **Preconditions**: SP-4 and SP-6 in daily use; Q-07 yes.
- **Work**: a design note first (owner approval), then push-to-talk, end-of-speech detection (VAD, MIT), a visible **confirmation or countdown before send** (so SI3 is weakened only by the user's explicit setting), barge-in, echo control (cancellation on or microphone muted while speaking), a hard guarantee that the loop cannot reach file actions, fill plans or any outward action (SI4), a visible state, a global stop, and CPU measurements on the practice PC.
- **Launcher**: `SP-7-conversation-mode.md` (an outline; refreshed before use).

### SP-8 — Prove it: validation, packaging, release gate

- **Work**: practice-PC performance re-run; the offline proof (firewall rule blocking the application and the sidecar; dictation and read-aloud still work; a download attempt fails); the packaged-app test procedure of SpeechLab M7 adapted; wiring of the sidecar and model manifest into the Windows installer (platform overlay or CI fetch step, per the SP-2 spike); CI on both platforms still green; credits view and notices (Q-09); the SI1 to SI12 table with evidence; update `docs/SPEECH.md`; release notes that say **Windows only, macOS not validated**.
- **Launcher**: `SP-8-prove-it.md`.

### MAC-HOLD (standing track, no phase)

While no Mac exists: write cfg-free code; keep CI compiling and unit-testing on `macos-latest`; do not claim macOS support; keep `docs/MACOS_VALIDATION.md` (SpeechLab) as the checklist; when a Mac is available, open a phase "Speech macOS" using that checklist and AssistantCabinetAI's own macOS work (nothing in either repository has ever been built on a Mac). Record in AssistantCabinetAI's decisions that this is a waiver, with the date and the owner.

---

## 6. Timing against the Knowledge Base programme

| KB lot | Needed by | Effect on speech |
|---|---|---|
| 0 merged, 1 in progress | — | none yet; SP-1 and SP-2 can start |
| 2 (normalisation, French phonetic key) | SP-5 | repair candidates by phonetic key |
| 3 (ports, packs, resolver) | SP-5, SP-6 | packs for `title_spoken` and the rules data format to imitate for `resources/speech` |
| 6 (query-time selection mapping) | SP-5 | selection-scoped surface forms |
| 7, 8 (interface, management dialog) | SP-4 merges | both edit `ChatPanel`/`Composer`-adjacent files, `ipc.ts`, catalogues; sequence SP-4's UI merge after them or resolve conflicts deliberately |
| 9 (surface, repair, guard) | SP-5 | the actual APIs |
| 11 (release gate) | SP-8 | `main` gets both programmes through `kb/integration` |

---

## 7. Per-phase artefacts and the step-by-step submission

1. Submit SP-0 (SpeechLab tab). Take the decisions. Run the probe on the practice PC.
2. Submit SP-1. Read `docs/SPEECH.md` and the decision table; accept or amend.
3. Submit SP-2, then SP-3, then SP-4; after each, run the human test and accept.
4. When KB lot 9 is merged, submit SP-5.
5. Submit SP-6; decide Q-07; submit SP-7 only if yes.
6. Submit SP-8.

Launchers and the master file are in `docs/prompts/integration/`. Their order and the tab names are in that folder's `README.md`.

## 8. Not in this plan

Streaming dictation; cloud anything; server-side speech; macOS validation; accent evaluation; word-level read-along; click-to-jump; voice cloning; changing the KB design; any change to SpeechLab beyond SP-0.
