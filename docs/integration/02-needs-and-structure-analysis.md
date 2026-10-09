# Integration analysis: needs, constraints and structure of the two projects

Written 2026-10-09 by the SpeechLab M8 follow-up session. Audience: the owner, and the agents who will run the speech
integration phases in AssistantCabinetAI tabs. English, like every developer-facing document of both repositories.

**Nothing in AssistantCabinetAI was modified to write this.** It was read (local clone, `git ls-remote` on the GitHub
remote) and every statement about it carries its source. Status words: **READ** = observed in a file at the commit named
in section 1; **INFERRED** = a conclusion drawn from READ facts; **NOT VERIFIED** = needs a test or a decision.

---

## 1. What was read, and at which state

| Item | State on 2026-10-09 |
|---|---|
| AssistantCabinetAI remote (`github.com/dekpo/AssistantCabinetAI`) | `ls-remote --heads`: `main` = `f27505f`, `kb/integration` = `d215939` (lot 0 merged, PR #20), `feat/kb-measure-and-unfreeze` = `ef149d5` |
| Local clone `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI` | on branch `feat/kb-store` with **uncommitted** lot 1 work (`knowledge/store.rs`, `tests/knowledge_store.rs`, edits to `error.rs`, `index_store.rs`, `knowledge/mod.rs`, locales, three docs). Read only; not touched |
| Knowledge Base programme | lot 0 merged; lot 1 code done (not committed, human test proposed); lots 2 to 11 todo. Source: local `docs/SESSION-KB-STATUS.md`, `docs/SESSION-KB-00-master.md` (git-ignored handoff files) |
| Files read | `AGENTS.md`, `.cursor/rules/*.mdc`, `docs/DECISIONS.md`, `docs/CHAT-UX-ASSESSMENT.md`, `docs/ROADMAP.md` (speech rows), `docs/HARDWARE.md`, `docs/PILOT-GP.md` (hardware lines), `docs/SESSION-KB-00-master.md`, `SESSION-KB-01-foundation.md`, `SESSION-KB-04-surfaces.md`, `SESSION-KB-STATUS.md`, `apps/desktop/src-tauri/{Cargo.toml,tauri.conf.json,capabilities/default.json,binaries/README.md,src/ocr/mod.rs,src/cancellation.rs,src/settings.rs,src/lib.rs}`, `apps/desktop/src/components/Composer.tsx`, `.github/workflows/ci.yml` |
| SpeechLab side | `docs/SPEECH_ENGINE_EVALUATION.md` (M8), `docs/DECISIONS.md` D-047, the source tree under `src-tauri/src/speech/` and `src/audio/`, `src/components/ReadAlong.tsx` |

The owner's statements of 2026-10-09 that frame this work are recorded in `01-owner-draft-and-review.md` and D-048.

---

## 2. AssistantCabinetAI facts that constrain the integration

| # | Fact (READ) | Source | Consequence for speech |
|---|---|---|---|
| F1 | Tauri 2 + React + TypeScript + Rust. "UI in TypeScript, never in Rust." Business code sits behind replaceable ports: `AIProvider` (`generate`, `embed`, `rerank`), `OcrProvider`, the index and the embedder | `AGENTS.md`; `src/ocr/mod.rs` | Speech gets its own ports, same style. `OcrProvider` is the template: id, version, one call per unit of work, machine-coded errors, degrade-to-unavailable |
| F2 | The webview never calls the server directly; every operation is a Tauri command; the allow-list, caps and no-store live in code | `AGENTS.md` | Dictation and read-aloud are Tauri commands; the webview captures audio and plays audio, nothing more |
| F3 | **Windows and macOS are both mandatory** for the client (19 Sept 2026): "an engine, library or runtime that exists on only one of them is disqualified". Platform differences live in the bundler config or in sidecar discovery, never in the capability code; no `#[cfg(windows)]` in `src/ocr/` | `AGENTS.md`; `docs/DECISIONS.md` (OCR table) | **Conflicts with the owner's macOS hold (D-048).** Both whisper.cpp and sherpa-onnx exist for macOS, so no engine is disqualified, but nothing was ever built or run on a Mac in either repository. The speech code must still be written without `cfg`; macOS validation is the part on hold. A recorded waiver is needed (decision Q-04) |
| F4 | CI runs `pnpm run build`, `vitest` and `cargo test --lib` on `windows-latest` **and** `macos-latest` | `.github/workflows/ci.yml` | Speech code must compile and unit-test on macOS even though nobody validates the product there. Anything that makes `cargo test --lib` need a Windows-only file breaks the macOS job |
| F5 | The Tesseract sidecar is **not declared** in `tauri.conf.json` (`bundle` has no `externalBin`) because `tauri-build` validates every declared path on every `cargo build/test`; the exact working block is recorded in `src-tauri/binaries/README.md` and goes back only together with a CI fetch step on both platforms. Sidecar DLLs must sit next to the sidecar executable. Discovery: `commands.rs` `sidecar_search_roots` (next to the executable, ancestors, `binaries/`) | `docs/DECISIONS.md`; `binaries/README.md`; `commands.rs` | A speech sidecar follows the same precedent. **Idea to verify (NOT VERIFIED):** Tauri 2 merges a platform file `tauri.windows.conf.json` into the configuration on Windows only, which would let the Windows installer declare `externalBin` without breaking the macOS CI job while macOS is on hold. Spike it first |
| F6 | **Voice rules already written**: cloud speech forbidden; audio never retained; a voice command never triggers `apply` on files (only a draft or a plan needing visual approval); `/v1/audio/*` reserved and answering 501; `webkitSpeechRecognition` rejected (cloud, absent in WKWebView) | `docs/DECISIONS.md` "Reserved contracts"; `docs/CHAT-UX-ASSESSMENT.md` items 6 and 7 | These become the invariants SI1 to SI4 (section 9). The reserved server routes should be recorded as "not used: speech is workstation-local" so nobody builds the LAN variant by accident |
| F7 | KB decisions D7 and D8 (8 Oct 2026): speech runs on the workstation; audio, transcripts and the KB never cross the network; **first model family Whisper (MIT), quality permitting; free licences only (Apache 2.0 / MIT), CC-BY only by an explicit later decision**; read-aloud is a button, off by default, "read answers automatically" a possible later addition. `SESSION-KB-01` item 5: "speech model licences get their own decision when the speech integration starts" | `docs/DECISIONS.md`; `SESSION-KB-00-master.md` | The recommended Parakeet model is CC BY 4.0: it needs that explicit decision (Q-01). Whisper (MIT) is the policy-compliant first engine |
| F8 | The 14 October 2026 milestone is withdrawn; voice moved from "frozen" to "planned, behind its own decision". Building speech needs a decision in `docs/DECISIONS.md` first | `DECISIONS.md` D9; `.cursor/rules/v0-sprint.mdc` | Phase SP-1 writes that decision before any code |
| F9 | **Language contract**: Rust returns machine codes, never prose; "a French string literal in `apps/server` or `src-tauri` is a bug"; interface strings are i18n keys in `src/locales/{fr-FR,en-US}.json` with a parity test; model-facing strings are profession-neutral | `AGENTS.md`; `SESSION-KB-00-master.md` I9, I10 | SpeechLab's French/English text rules (normaliser, splitter) cannot be copied as Rust source (section 4) |
| F10 | `apps/desktop/src/guards/sources.test.ts` (run by `pnpm run test`) scans **every `.rs` under `src-tauri/src`**, test modules included, and fails on any non-ASCII character and on any line containing one of `le la les des une est erreur fichier dossier envoi veuillez aucune` | `SESSION-KB-STATUS.md` note "Lot 0 -> every Rust lot" | Measured in SpeechLab: `normalise.rs` has 111 lines with non-ASCII and 35 with marker words, `postcorrect.rs` 23 and 17, `sentences.rs` 21 and 12, `critical.rs` 12 and 6, `numbers.rs` 6 and 5. None of that code can be moved as it is |
| F11 | Lexicon **packs are data** (`resources/knowledge/…`, `resources/tabular-questions/…`): they may hold words, never user-visible sentences. The KB plans a `title_spoken` pack table ("dr" to "docteur") and a `spoken_form` attribute | `SESSION-KB-01-foundation.md`; `SESSION-KB-04-surfaces.md` | The right home for French number words, units and abbreviations is a speech pack (JSON), not Rust code |
| F12 | **Content Security Policy**: `default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost`. No `blob:` anywhere. Capabilities: `core:default`, `dialog:allow-open`, `clipboard-manager:allow-write-text` | `tauri.conf.json`; `capabilities/default.json` | SpeechLab learned (D-045) that its AudioWorklet loads from a `blob:` URL and was blocked by an equivalent policy; playback of generated audio needs `media-src` for `blob:`/`data:`. The policy must be extended (or the worklet shipped as a static file, which is the tighter choice) |
| F13 | Settings: `settings.json` in the platform config folder, `serde(default)`, clamped on the way out; "server URL, model and work folder are configuration, never constants"; an "Advanced" group exists in `SettingsDialog.tsx` | `settings.rs`; `AGENTS.md` | Speech options are settings with named default constants |
| F14 | `Composer.tsx`: the draft is **held by the chat panel** (`draft`, `onDraftChange`); send is explicit (Enter or button); there is a model selector. `MessageList.tsx` has the per-answer action row (copy, edit, regenerate); answers are rendered as Markdown through `react-markdown` | `Composer.tsx`; `MessageList.tsx`; `DECISIONS.md` | Dictation writes into the draft through `onDraftChange` and never calls `onSend`. Read-aloud is one more action in the row, and the text to speak must be derived from the Markdown content, not read off the screen |
| F15 | `cancellation.rs`: one run in flight, a `watch` channel, the run's future is dropped on stop; `tokio` features are `macros`, `sync`, `time` only | `cancellation.rs`; `Cargo.toml` | The same pattern serves a sidecar run (drop the future, kill the child). Spawning processes needs `tokio`'s `process` feature or `std::process` on a blocking worker (as SpeechLab did) |
| F16 | **The AI server is a Mac mini M5 Pro (64 GB) at the practice; the pilot workstation is a Windows PC installed in 2019, specification unknown** ("the project owner installed it and can check it herself") | `docs/HARDWARE.md`; `docs/PILOT-GP.md` | The single biggest performance unknown (section 6). SpeechLab's figures come from a 2024 laptop CPU |
| F17 | The installer track (Windows) is gated by the DPIA, a named data controller and disk encryption, not by the KB; the installer is unsigned and Tesseract is not yet bundled | `DECISIONS.md` D9; `binaries/README.md` | Speech packaging joins that track; a model file is the heaviest thing it would carry |
| F18 | **Branch model of the KB programme**: lot branches start from `kb/integration` and merge back by pull request; `main` stays the untouched baseline until the lot 11 release gate | `SESSION-KB-00-master.md` section 0 | The owner's draft says "examine the `main` branch": `main` is deliberately frozen. Speech branches should start from `kb/integration` (decision Q-03) |
| F19 | Shared hot-spot files: `commands.rs` (1,823 lines), `lib.rs`, `src/lib/ipc.ts` (837), `settings.rs`, `SettingsDialog.tsx`, the two locale files, `error.rs`. The KB notes already warn that lots 8, 9 and 10 "share `commands.rs`, `ipc.ts` and the catalogues, so expect a small merge" | `SESSION-KB-STATUS.md` | Speech code goes in **new files** (`src-tauri/src/speech/`, `src/speech/`, `src/audio/`); shared files get the smallest possible registration edits, sequenced against KB lots 7 to 9 |
| F20 | Tests and process: `cargo test`, `pnpm run test`, `pnpm run build` must stay green; each lot ends with a lot report, a proposed human test and an updated status file; git is owner-only; **no AI attribution anywhere git-facing**; local handoffs are `docs/SESSION-*.md` (not versioned) | `AGENTS.md`; `SESSION-KB-WORKFLOW.md` | The speech programme reuses the same ritual (master prompt, section 11) |

---

## 3. What SpeechLab brings (evidence, not code)

Full evidence: `docs/SPEECH_ENGINE_EVALUATION.md`. What matters for the integration:

1. **Speech-to-text, one speaker, CPU, 2024 laptop (about 20 % background load)**: Parakeet TDT v3 int8 WER 2.3 %, RTF about 0.10, 1.6 to 1.9 GB, 7 to 10 busy cores, CC BY 4.0, no cancellation. whisper.cpp small q5_1 WER 3.9 % (5 beams), **RTF 1.1 to 1.3 (slower than real time)**, 0.4 to 0.5 GB, MIT, real cancellation. whisper.cpp base 8.4 %, RTF 0.29 to 0.34. [C-01 to C-07]
2. **Names and drug names**: every engine misspells drug names (best 4 of 6). Strict dictionary post-correction fixed more with 0 words broken; the whisper.cpp initial prompt gave a gain on small and base (3.9 to 3.3 %) but no reliable gain on drug names, cost 25 to 40 % more time and hurt sentences without key terms; Parakeet hotwords needed a surrogate vocabulary and regressed at a high score. [C-17 to C-20]
3. **Text-to-speech**: Piper through sherpa-onnx is 13 to 29 times faster than real time; English voices `ljspeech`, `kristin` (public domain) and `libritts_r` (CC BY 4.0) are judged good by the owner; the French voices judged good (siwis, gilles) have a restricted lineage; Coqui css10 (BSD-3-Clause) is the only clean French voice, 3 out of 5, and speaks digits only with the rewriting on. [C-21 to C-30]
4. **A text rewriting layer** (numbers, dates, times, units, abbreviations to words, fr and en) and a **sentence splitter with measured segments** exist and are tested in Rust; a read-along highlight exists in React.
5. **Platform**: Windows 11 packaging, microphone permission in the release origin, strict CSP, offline proof per program were verified; macOS is NOT VERIFIED and now **on hold** (D-048).
6. **Licensing**: the full sherpa-onnx libraries contain the GPL-3.0 phonemizer espeak-ng (95 occurrences in the packaged executable); NeMo models are CC BY 4.0; no `LICENSE` file in SpeechLab; the legal reading list is in report section 9.6.
7. **Measurement tooling**: `bench` (accuracy, speed, memory, preflight), `tts` (speed), UI test scripts over the DevTools protocol, the offline-proof firewall scripts. These can be run on the practice PC.

---

## 4. Reuse map: what to keep, rewrite or leave behind

The rule of the owner: **do not copy everything**. Decision per SpeechLab asset:

| SpeechLab asset | Verdict | Why and how |
|---|---|---|
| Rust traits `SpeechToTextProvider`, `TextToSpeechProvider`, `Capabilities`, the cancel token (`speech/provider.rs`, `types.rs`) | **Rewrite, same shape** | The shape is proven; the names, error type and cancellation follow AssistantCabinetAI (`OcrProvider` style, `AppError` machine codes, `cancellation.rs`) |
| `whisper_cpp.rs` (external `whisper-cli`, timing parse, kill on cancel) | **Port** | Closest to the existing Tesseract sidecar. Drop SpeechLab's timing-report parsing needs only if not wanted; keep: args building, cancellation by kill, no-console-window on Windows, locate-sidecar order |
| `sherpa.rs` + `models.rs` families (NeMo transducer, Canary, Whisper) | **Later, in a separate worker** | Only if the owner accepts CC BY 4.0 (Parakeet) and the practice-PC probe needs it. Never linked into the main executable (GPL risk of the full libraries, 1.9 GB resident) |
| `tts.rs` (per-sentence synthesis, segments, cancel) | **Later, in a separate TTS sidecar** | Needs the full sherpa-onnx libraries (espeak-ng, GPL-3.0): isolate it in its own executable, or use the system voices (section 7) |
| `normalise.rs`, `sentences.rs` | **Rewrite as data-driven** | The algorithm (cardinals, ordinals, times, dates, units, phone numbers, plural rules; abbreviation-aware splitter) is valuable and tested (15 and 11 unit tests), but its words are French/English literals. Re-implement the engine in ASCII Rust reading `resources/speech/{fr-FR,en-US}.json` packs (number words, units, months, abbreviations, `title_spoken`). The old unit cases become fixtures in `src-tauri/tests/` (not scanned) |
| `postcorrect.rs` (strict, medium, loose dictionary correction) | **Do not copy. Use as the specification of the KB `repair` thresholds** | The KB plan has a better design (`repair::propose` returns proposals with `Suggest`/`Ask`, never rewrites silently, selection-first). SpeechLab supplies the evidence: STRICT rules (6+ letters, same first letter, 1 edit up to 11 letters) broke 0 words; LOOSE is a regression; the look-alike hazard (prednisolone/prednisone) is the reason for never replacing silently (I-045) |
| `chunking.rs`, `vad.rs` (Silero) | **Optional, later** | Short audio (25 s or less) is provably unchanged, so dictation needs none of it. Useful for hands-free end-of-speech detection (conversation mode) and for dictations over 30 s with sherpa Whisper only. Silero is MIT, 0.6 MB |
| `metrics.rs`, `numbers.rs`, `critical.rs`, `benchmark.rs`, `dataset.rs`, `termstudy.rs`, `probe.rs` | **Leave in SpeechLab** | Evaluation tooling. AssistantCabinetAI already has `number_check.rs` for answers. `bench` stays the instrument to measure engines on the practice PC |
| `download.rs`, model manifest design (id, size, SHA-256, licence, sourceUrl, platforms, packaging kind) | **Port the design** | AssistantCabinetAI has no client-side model manager (Ollama models live on the server; Tesseract data is bundled). The checksum-pinned manifest and the "marker file written after verification" idea carry over. Rule from `AGENTS.md`: every pulled model needs its row in `models/LICENSES.md` |
| TypeScript `src/audio/recorder.ts` (getUserMedia + AudioWorklet, processing flags off, 16 kHz mono), `wav.ts` (encoding, `signalStats` level and clipping check) | **Port** | Pure webview code, already verified on WebView2 including the release origin. Adjust: no clip store, no file save; audio goes to a Tauri command and is dropped. Ship the worklet as a static file to avoid `blob:` in the CSP |
| `ReadAlong.tsx`, `readAlong.ts` (active segment from the player clock, auto-scroll) | **Defer** | The owner wanted a sentence highlight, but AssistantCabinetAI renders answers as Markdown (`react-markdown`), so mapping sentence spans onto the rendered tree is a different problem from SpeechLab's plain text. First version of read-aloud: play and stop, no highlight; the pure functions are reused if a plain-text read-along view is added |
| Clip store (`clips.rs`), dataset recorder, comparison panel, model tables | **Leave** | Lab features. Product rule: audio is never retained |
| `scripts/offline_proof_*`, `ui_test_*.mjs`, `cdp.mjs` | **Adapt as test tooling** | The firewall proof and the DevTools driving are reusable for the speech release gate; AssistantCabinetAI's origin and port differ |
| Result folders and `docs/` | **Reference** | Cited by path from the new documents; not copied |

---

## 5. Target structure

### 5.1 Principles

1. **Autonomous module.** A `speech` module with no dependency on chat, retrieval or the gateway. Its only links to the rest of the product: text in (the composer draft, the answer text), text out (the transcript), and **read-only** KB hints. Removing it (setting off, or deleting the folder) leaves the application unchanged.
2. **Engines out of process.** The main executable never links a speech engine. Reasons (INFERRED, each from a READ or measured fact): the full sherpa-onnx libraries contain GPL-3.0 code (so linking them into the product's executable is the worst licensing outcome); Parakeet holds 1.6 to 1.9 GB; a sidecar can be killed (real cancellation, F15); a crash does not take the application down; hints and text can travel by stdin rather than by the command line.
3. **Two ports, two engines.** Speech-to-text and text-to-speech are independent (the owner's draft says so); they share only the settings group, the model manifest and the error taxonomy.
4. **Capture and playback stay in the webview.** `getUserMedia` and the audio element exist on both WebViews (F2, F3); Rust receives PCM, never owns the microphone.
5. **Whole-clip, not streaming.** SpeechLab evaluated whole clips only. The first user experience is "press, speak, press, wait, edit". Partial results are NOT EVALUATED (C-53).
6. **Local only, and provably.** No network code in the speech module except an explicit model installer; the offline proof of SpeechLab is re-run at the release gate.

### 5.2 Layout (proposed)

```text
AssistantCabinetAI/
  apps/desktop/src-tauri/
    src/speech/                      NEW (all ASCII, no French literal, no cfg)
      mod.rs                         public types, SpeechError machine codes
      stt.rs                         trait SpeechToTextProvider (+ SttRequest, SttResult, SttAvailability)
      tts.rs                         trait TextToSpeechProvider (later phase)
      hints.rs                       trait SpeechHintSource (adapter over knowledge::surface / repair; KB off => empty)
      whisper_cli.rs                 adapter: whisper.cpp sidecar (first engine)
      worker.rs                      adapter: speech-worker sidecar (second engine, optional, later)
      fake.rs                        deterministic fake provider for tests
      temp_audio.rs                  private temp WAV with guaranteed removal (SI1)
      speakable.rs                   text for speech: Markdown and citations stripped, rules from packs (later phase)
      manifest.rs                    speech model manifest + verification (no download code in v1)
    resources/speech/                NEW data packs
      models-manifest.json           ids, sizes, SHA-256, licence, source
      rules/{fr-FR,en-US}.json       number words, units, months, abbreviations, title_spoken
    binaries/                        sidecars (not committed; fetch script; same README precedent)
    tests/speech_*.rs                integration tests (not scanned by the language guard)
  apps/desktop/src/
    audio/{recorder.ts,levels.ts,wav.ts}   ported capture code (+ static worklet file in public/)
    speech/{ipc.ts,useDictation.ts,useReadAloud.ts}  state machines, no engine names
    components/{MicButton.tsx,DictationStatus.tsx,ReadAloudButton.tsx}  NEW; Composer/MessageList get one-line mounts
    locales/{fr-FR,en-US}.json       speech.* keys (parity test)
  apps/speech-worker/                OPTIONAL, later: separate Cargo crate hosting sherpa-onnx (own licence file)
  scripts/fetch-speech-resources.ps1 (+ .sh stub)  checksum-pinned downloads into the git-ignored trees
  docs/SPEECH.md                     canonical spec (created in phase SP-1)
```

SpeechLab keeps its repository, frozen (D-048). It is the reference and the measurement bench, not a dependency.

### 5.3 The two ports (sketch, to be fixed in SP-1)

```text
SpeechToTextProvider
  id(), version()                          stable identity, stored nowhere persistent (transcripts are not stored)
  availability(locale) -> Availability     Ready | ModelMissing | EngineMissing | Unsupported(locale)   machine codes only
  transcribe(SttRequest, cancel) -> SttResult
      SttRequest { pcm_16k_mono: &[i16] or temp wav handle, locale: "fr-FR"|"en-US", hints: &[HintTerm] (<= N), max_seconds }
      SttResult  { text, audio_ms, processing_ms, engine_id, engine_version }       no per-word confidence needed in v1
  errors: speech_engine_unavailable, speech_model_missing, speech_audio_too_short, speech_audio_too_long,
          speech_cancelled, speech_engine_failed, speech_locale_unsupported

TextToSpeechProvider                       (phase SP-6)
  voices(locale) -> Vec<Voice { id, display_key, gender?, local: bool }>
  synthesize(SpeakableText, voice, speed, cancel) -> AudioHandle (WAV bytes in memory or private temp) + optional segments
  errors: tts_voice_unavailable, tts_engine_unavailable, tts_cancelled, tts_text_too_long

SpeechHintSource                           (read-only; empty when the KB mode is off)
  hints(selection, locale, limit) -> Vec<HintTerm { display, spoken_form?, weight }>      via knowledge::surface_forms(purpose = SpeechHints)
  propose_repairs(transcript, selection, locale) -> Vec<RepairProposal>                 via knowledge::repair::propose
```

`HintTerm` carries display forms only; **no entity ids and no KB text leave the process boundary except the hint strings handed to the engine** (section 8).

### 5.4 Dictation flow

```text
[Mic button] -> webview: getUserMedia + AudioWorklet -> 16 kHz mono PCM, level/clipping check
   -> invoke speech_transcribe(pcm, locale) ------------------------------------------------> Tauri command
        Rust: availability check -> temp audio (private folder, removed in Drop/finally/startup sweep)
              -> hints = SpeechHintSource (KB mode != off, user setting on) -> provider.transcribe(...)
              -> temp audio removed -> result text
   <- { text } or machine code
 webview: insert into the composer draft (cursor position), mark "dictated", show repair chips (KB phase)
 user edits -> user presses Send (explicit).  Dictation never calls send; never touches the KB (SI3, SI5).
```

### 5.5 Read-aloud flow (later phase)

```text
[Read aloud on an answer] -> Rust: speakable text from the answer's Markdown source (citations and links removed,
   numbers/dates/units spoken by pack rules, spoken forms from the KB for names in the cited sources)
   -> TextToSpeechProvider (system voice local-only guard, or TTS sidecar) -> audio -> webview audio element
 Stop button; nothing plays by itself unless the user enabled "read answers automatically" (D8).
```

---

## 6. Performance and hardware: the unknown that decides the engine

- SpeechLab's measured machine: Core 7 150U (2024), 23.6 GB, about 20 % background CPU. On it whisper.cpp **small is slower than real time** (a 4.4 s sentence took 5.4 to 5.9 s) and base is 3 times faster than real time at 8.4 % WER. Parakeet is 10 times faster than real time but needs 1.6 to 1.9 GB.
- The pilot workstation is a **Windows PC from 2019, specification unknown** (F16). It is almost certainly slower than the SpeechLab laptop for this workload. INFERRED, not measured.
- Therefore: **no engine or model size should be fixed before the practice-PC probe** (phase SP-0): run SpeechLab's own `bench` (or a small wrapper) on that PC with the candidate models and record RTF, peak memory, busy cores and the background load. Budgets are proposals for the owner to accept: dictation of a 10-second utterance returns text in at most about 10 s (RTF 1.0), extra resident memory at most about 1 GB (or the owner's call), no freeze of the interface.
- Decision tree for the first engine (full text in the plan, section "Engine decision"): if base or small meets the budget on the practice PC, ship **whisper.cpp** (MIT, existing sidecar pattern, D7); if neither does and Parakeet does, the owner decides on CC BY 4.0 (Q-01) and the 1.9 GB; if nothing does, dictation waits for a hardware change or for the speech model to run on the server (which reopens D7, because audio would cross the LAN; the Mac mini M5 Pro would run Whisper large quickly, but this is the owner's rule, not an engineering preference).
- Concurrency on the workstation: indexing, OCR and embedding calls already load the CPU during an Analyse pass; a dictation during an Analyse is a realistic overlap (NOT VERIFIED). The speech sidecar runs at normal priority and the interface must stay responsive; add a "busy" state rather than queueing silently.
- Optional optimisation, NOT EVALUATED: transcribe each utterance as soon as silence is detected (the VAD of SpeechLab) so the wait after the last word is shorter.

---

## 7. Text-to-speech: three paths, one decision

| Path | For | Against | Status |
|---|---|---|---|
| A. Operating-system voices through `window.speechSynthesis` (WebView2: SAPI; WKWebView: AVSpeechSynthesizer) | No sidecar, no model, no GPL, both platforms natively (`CHAT-UX-ASSESSMENT.md` item 6) | Windows also lists **online** neural voices that send text to a third party: the code must keep only `voice.localService === true` and refuse (never fall back) when no local voice matches the locale, with a test; availability depends on the installed language packs; `getVoices()` is asynchronous; no speed/pause control equivalent to SpeechLab's; **quality was never evaluated in SpeechLab**; no segment timings | NOT EVALUATED here. A short listening spike (SAPI French voice against Piper siwis, coqui) is the first task of the TTS phase |
| B. Piper through a TTS sidecar built on sherpa-onnx | Measured: 13 to 29 times faster than real time, the owner's preferred voices, speed control, segments, normaliser-compatible | Contains GPL-3.0 espeak-ng: only acceptable in a **separate executable** with its own notice (legal reading open), or after a clearance; French voices rated good are restricted; voice packages must be shipped or downloaded | Verified in SpeechLab; licensing open |
| C. Coqui css10 (character-based, BSD-3-Clause) | Needs no phonemizer, clean licence | 3 out of 5, mispronounces; still inside the sherpa libraries unless a build without the phonemizer exists (I-054) | Fallback for French only |

Recommendation: **start with path A for a first read-aloud** (zero licensing cost, quickest to verify, honours D8), **measure it against Piper by ear** in the same phase, and add path B behind the same `TextToSpeechProvider` port only if the owner finds the system voices insufficient and accepts the licensing conditions. This is a change from a "Piper first" reading of the SpeechLab report and is driven by the AssistantCabinetAI constraints (F3, F7).

---

## 8. The Knowledge Base as the shared vocabulary

### 8.1 What the KB already promises (READ)

`surface_forms(selection, limit, purpose)` returning ranked `{entity_id, display, normalized, phonetic_key, spoken_form, type_id, subtype, weight}`; `repair::propose(transcript, selection, locale, case_reliable)` returning spans with candidates and an action `Suggest | Ask | None`, selection-first, never rewriting text; a `spoken_form` attribute (manual or from pack `title_spoken`); a contract fixture `tests/fixtures/knowledge/repair-fr.json` with `heard -> expected` pairs and must-not-change pairs; invariants I3 (a dictation never writes to the KB), I7 (entities outside the selection are never revealed), I8 (nothing the KB stores is sent to a model or logged as text), I9 (no KB string in a model-facing text). Lots 2 (normalisation and French phonetic key), 3 (ports, packs, resolver) and 9 (surface API, repair, guard) are the dependencies; none exists yet (lot 1 only).

### 8.2 How speech uses it, with the SpeechLab evidence

| Need | Mechanism | Evidence and design rule |
|---|---|---|
| Recognise proper names, organisations, drug names better | **Repair-first**: transcribe, then `repair::propose` on the text; show `heard -> display` chips the user accepts or ignores | Strict dictionary correction was the only technique with 0 broken words; engine biasing was marginal or harmful (C-17 to C-20). Repair also keeps hint text out of any process command line |
| Engine hints (optional, second step) | `surface_forms(purpose = SpeechHints)` limited to the selection and to a short list | whisper.cpp prompt: at most about 224 tokens (`--prompt`, max `n_text_ctx/2`); it **raises inference time by 25 to 40 %** and lowers accuracy slightly on sentences without key terms; helped small and base, hurt tiny. Parakeet hotwords need a SentencePiece vocabulary the download lacks (I-043). Therefore hints are an experiment behind a setting, off by default, measured on the practice PC |
| Correct spelling of what was heard | The transcript is shown as text and the KB `display` form replaces the phonetic guess only on acceptance | SpeechLab I-045: a dictionary cannot tell a misspelling from a rare real word; in a medical text nobody re-reads a plausible word. Hence `Suggest`/`Ask`, never silent replacement, for the `item/medication` and `person` types |
| Pronunciation of names in read-aloud | `spoken_form` lookup order: manual attribute, then pack `title_spoken`/rules, then the engine's own phonemizer | The engines accept text only (no phoneme injection through the sherpa-onnx API: NOT VERIFIED for every voice): a **respelled text** ("cé pé a aime") is the portable mechanism and works for phonemizer and character voices. The displayed text is never changed |
| Numbers, dates, units in read-aloud | Rules from `resources/speech/rules` (the SpeechLab normaliser, rewritten data-driven) | 0/36 to 28/36 and 30/36 key numbers heard on the Coqui voice (C-28); the owner uses the rewriting ON for all voices (D-043) |

### 8.3 Privacy consequences of using the KB (new, found in this analysis)

1. **Names of people in a hint list travel to the engine.** With the `whisper-cli` sidecar the prompt is a **command-line argument** (verified: `whisper-cli --help` has `--prompt` and no prompt-file option), visible to other processes of the same user and to any tool that logs process creation. KB invariant I8 forbids logging KB text; a command line is not a log, but it is a leak path that the pilot's DPIA would have to mention. Mitigations, in order of preference: do not use hints with `whisper-cli` (repair-first); use hints only with an engine that takes them through an API or stdin (the future worker); keep the list short and selection-scoped; make the setting opt-in with a plain explanation.
2. **A dictated transcript contains names**; it lives in the composer and the conversation, like typed text. It must never reach a log or a timing line (SI6) and never the KB (SI5).
3. **Temp audio**: whisper-cli needs a file. A private folder under the app-local data directory, files removed on success, error, cancel and by a startup sweep; the existing disk-encryption gate (F17) covers the residue risk. Test it (SI1).
4. **Read-aloud in a practice**: spoken answers can be overheard in a waiting room, the same concern as KB decision D10 on screen. Default off, a visible speaker state, no automatic reading of an answer unless the user opted in (D8).

### 8.4 Order dependency

Dictation without the KB can ship first. KB-assisted spelling needs KB lots 2, 3, 6 and 9 merged into `kb/integration` (plan phase SP-5). Nothing in the first phases may import `knowledge::*` beyond the trait `SpeechHintSource`, which defaults to an empty implementation.

---

## 9. Invariants for the speech programme (proposed; each gets a named test)

| ID | Invariant | Test (suggested name) |
|---|---|---|
| SI1 | Audio is never retained: temp files are removed after success, error, cancel and by a startup sweep; no audio in settings, index, logs or clip store | `temp_audio_is_removed_on_success_error_and_cancel`, `startup_sweep_removes_orphan_audio` |
| SI2 | No network from the speech module. Speech code imports no HTTP client; the only network use, if any, is an explicit model installer | `speech_module_has_no_http_client_dependency` (source scan), offline proof at the gate |
| SI3 | A transcript only fills the composer draft. Nothing is sent to the model without the user's explicit send (conversation mode, if ever built, is a separate opt-in with its own decision) | `dictation_never_calls_send` (vitest) |
| SI4 | Voice never confirms or applies a file action, a fill plan or any outward action | `no_speech_path_reaches_apply` (review + vitest on the action components) |
| SI5 | Dictation never writes to the KB (extends KB I3) | `dictating_does_not_change_the_knowledge_base` (row-hash snapshot) |
| SI6 | No transcript, hint or KB text in any log or timing line; counts and milliseconds only | `speech_timing_line_holds_no_text` |
| SI7 | Read-aloud uses only local voices or engines; a non-local system voice is never selected and never a fallback | `a_non_local_voice_is_never_selected` |
| SI8 | The spoken text is derived by code from the displayed answer; the model never produces it | `speakable_text_is_not_model_facing` |
| SI9 | Any speech failure degrades to typed chat with a machine code; speech never blocks launch | `missing_engine_reports_unavailable_and_chat_still_works` |
| SI10 | `src-tauri/src/speech/` is ASCII, holds no French marker word, no `#[cfg(windows)]`; every user-visible sentence is an i18n key with fr/en parity | the existing `sources.test.ts` guard and `catalogues.test.ts` |
| SI11 | Cancel works at every stage: recording, transcribing (child killed), synthesising, playing | `cancelling_a_transcription_kills_the_child_and_removes_the_temp_audio` |
| SI12 | The KB hint path is read-only and selection-scoped (extends KB I1, I7); with KB mode `off` the hint list is empty | `hints_never_leave_the_selection`, `kb_off_gives_no_hints` |

---

## 10. Interface design notes (for SP-4 and SP-6)

- **Mic button** in the composer footer, next to the model selector. States: `unavailable` (hidden or disabled with a localised hint when the engine or model is missing), `idle`, `requesting-permission`, `recording` (level meter, elapsed time, stop), `transcribing` (spinner, cancel), `done` (text inserted, brief note), `error` (machine code to localised sentence). Keyboard: a shortcut to toggle; focus returns to the textarea; `aria-live` status for the state (accessibility is not in the owner's draft and is a cheap addition).
- **Insertion**: at the cursor, replacing a selection; one undo step returns to the previous draft; never auto-send. A short dictated text shows the usual composer; a very quiet or clipped recording shows the level warning SpeechLab built (peak, share of clipped samples).
- **Permission**: WebView2 asks once per origin (`tauri.localhost` in the release build, the dev server origin in development) and remembers it (VERIFIED in SpeechLab M7); denial is a machine code with a pointer to the system setting. macOS needs `NSMicrophoneUsageDescription` and an entitlement (ON HOLD).
- **Settings group "Speech"** (inside the existing Advanced group at first): speech on/off, engine (shown only when more than one is installed), language (default from the locale), name hints on/off, read-aloud voice and speed, "read answers automatically" (off), "spoken conversation" (off, later). Defaults are named constants in `settings.rs`.
- **Read-aloud**: a button in the answer action row beside copy/edit/regenerate; stop; a small speaking indicator; speed from settings. No highlight in v1.
- **Strings**: every sentence in both catalogues; the model gets none (the speech module adds no model-facing text).

---

## 11. Test strategy (AssistantCabinetAI side)

1. Unit: speakable-text rules from fixtures derived from SpeechLab's unit cases; temp-audio lifecycle; manifest verification (SHA-256); availability matrix; hint truncation; error-code mapping.
2. Contract: a fake provider (deterministic) drives every integration and UI test; a real-engine test (`#[ignore]`, needs the sidecar and a model) transcribes a short synthetic clip.
3. Invariants SI1 to SI12 as named tests.
4. Frontend (vitest): state machine of dictation (all transitions, cancel at every state), insertion rules, no send, catalogue parity, error mapping.
5. Human tests proposed at the end of each phase (the KB workflow already requires this): with the **pilot GP's own voice and microphone** on fictional fixtures (`fixtures/gp-sandbox/`), never real patient data.
6. Platform and packaging: CI keeps compiling and unit-testing on both platforms; the Windows installer test reuses SpeechLab's packaged-app procedure (silent install, CDP-driven checks, offline proof by firewall rule). macOS: on hold.
7. Performance: the probe on the practice PC and a repeat at the release gate; figures quoted with background load and the 10 to 20 % margin SpeechLab measured.
8. Accuracy: a small acceptance set of fictional sentences with names and drug names from the KB sandbox, read by the pilot GP; **no accent claim** (SpeechLab dropped accents, D-046; the only speakers that count for the product are the real users). Quote WER only with its interval and the number of speakers.

---

## 12. Risks

| Risk | Why it matters | Mitigation |
|---|---|---|
| The 2019 PC cannot run an accurate engine in acceptable time | Dictation unusable | Probe first (SP-0); decision tree; keep typed chat as the default; consider smaller model + repair |
| CC BY 4.0 versus the free-licence policy | Best engine is not yet allowed | Q-01 decision; whisper.cpp as policy-compliant first adapter |
| GPL espeak-ng reaches the product | Distribution obligations | Out-of-process TTS sidecar or OS voices; no sherpa full libs in the main executable |
| macOS never built | Violates "both mandatory" | Recorded waiver Q-04; `cfg`-free code; CI keeps compiling; checklist `docs/MACOS_VALIDATION.md` stays |
| Source guard blocks French word rules | Cannot copy the normaliser | Data-driven packs (section 4) |
| CSP blocks the worklet or playback | Microphone/audio fail in the release build | Static worklet file; `media-src` extension decided and tested in SP-4 |
| Hint list leaks names via command line | Breaks the spirit of I8 | Repair-first; hints opt-in; worker with stdin later |
| Merge conflicts with KB lots 7 to 10 | Delays both programmes | New files; minimal edits to shared files; sequence UI wiring after lot 8/9 or on `kb/integration` |
| Transcription mistakes on numbers and doses | Clinical-adjacent harm | The user edits and sends; critical-number warnings are NOT in the product yet: consider reusing `number_check` ideas on the dictated text (decision later); never auto-send |
| Conversation mode auto-sends dictated text | Contradicts SI3 and the explicit-send rule | Separate opt-in phase with its own decision and design (confirmation, barge-in, echo control) |
| Model distribution without Internet | Practice machines may be offline; "no mandatory Internet" | Decision Q-05: installer with model, side-load folder, or LAN mirror of model files (never audio) |
| Scope creep from the vision | A speech feature grows into a voice assistant | Phases are separable; stop after any phase is green |

---

## 13. Open owner decisions (summary; full table in the plan)

Q-01 CC BY 4.0 models allowed for speech? Q-02 process model (sidecar worker, no engine in the main executable). Q-03 base branch (`kb/integration` versus `main`). Q-04 macOS waiver wording. Q-05 how model files reach the practice PC. Q-06 TTS path order (system voices first) and the French interim. Q-07 conversation mode: build or not, and when. Q-08 acceptance budgets (latency, memory) after the probe. Q-09 licence of the code and credits view.
