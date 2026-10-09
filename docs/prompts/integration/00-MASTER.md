# Speech integration programme: master brief (read first in every Speech tab)

> **GATE (D-050): ON HOLD.** On 2026-10-09 the owner decided to finish the Knowledge Base first and then to relaunch a pass on the
> integration plan to adapt it. Read `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\STATUS.md` before anything else.
> **If it does not say `STATUS: RESUMED`, no speech phase may run**: stop and tell the owner, in French, that the integration waits
> for the end of the Knowledge Base and for the re-baseline pass (`SP-R-rebaseline.md`). The only exceptions: the re-baseline pass
> itself, and SP-0 when the owner explicitly asks for it. Everything below is **draft v1, valid as of 2026-10-09**: its facts about
> AssistantCabinetAI will have changed by the time the work resumes. Prerequisites the Knowledge Base must keep:
> `docs\integration\KB-PREREQUISITES.md`. Note to drag into KB tabs: `KB-TAB-NOTE.md` (this folder).

English, like every developer-facing document of both repositories. The owner speaks French: **answer the owner in French in
the chat; write every file, comment and commit message in English.**

This file is the contract. A phase launcher (`SP-n-*.md`) says *what to do now*; this file says *what must always be true*.
It was written on 2026-10-09 from the analysis in `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\`. If a fact below
has changed since (a KB lot merged, a decision taken), the repositories win: re-check before relying on it.

## 0. How to use

- One phase = one chat tab. The tab name is the phase's title (see `README.md` of this folder).
- Drag into the new tab: **this file, the phase launcher, and the three reference documents** below. Nothing else is needed; the
  launcher lists what to read.
- A phase starts only when the owner says the previous one is accepted. Read the status file first (section 8).
- Never start a phase silently out of order. If a precondition in the launcher is not met, **stop and say so**; do not work around it.

## 1. Mission

Integrate **workstation-local speech** into AssistantCabinetAI: dictation (speech-to-text, STT) into the chat composer, then
read-aloud of answers (text-to-speech, TTS), with the Knowledge Base (KB) improving the understanding, spelling and
pronunciation of proper names, organisations and medicines. SpeechLab (frozen) is the evidence and the toolbox; the product is
AssistantCabinetAI. Speech is an **autonomous, switchable module**: it never mixes with the conversational core, and removing
it leaves the product unchanged.

## 2. Decisions already taken (do not reopen without the owner)

1. **SpeechLab is finished and frozen** (its decision D-048, 2026-10-09). Only phase SP-0 edits it. Evidence: `docs/SPEECH_ENGINE_EVALUATION.md`.
2. **macOS is on hold until further notice** (the owner has no Mac). Speech code is still written without `cfg` and must compile and unit-test in the existing CI job on `macos-latest`; **nothing is validated on macOS and no macOS claim is made**. The AssistantCabinetAI rule "Windows and macOS both mandatory" is waived for speech validation only, by a decision to be recorded in SP-1 (Q-04).
3. **Everything local.** Audio, transcripts and the KB never cross the network, not even the practice LAN. Cloud speech is forbidden. The reserved server routes `/v1/audio/*` stay unused.
4. **The user sends explicitly.** A transcript fills the composer and is editable. (A conversation mode that auto-sends is a separate, optional phase with its own decision.)
5. **Voice never triggers a file action** (apply, rename, fill plan, move, delete) and never confirms an outward action.
6. **Models: free licences.** KB decision D7: Whisper (MIT) first; Apache 2.0 and MIT only, CC BY only by an explicit later owner decision (Q-01). Every speech model gets its row in `models/LICENSES.md`, read from the exact card, before it is proposed or pulled.
7. **Engines run out of process** (sidecar), never linked into the main executable (GPL phonemizer in the full sherpa-onnx libraries; memory; crash isolation; real cancellation).
8. **Two ports**: `SpeechToTextProvider` and `TextToSpeechProvider`, independent. Capture and playback are webview code.
9. **Repair-first for names**: transcribe, then propose KB corrections the user accepts; never a silent replacement; hint lists to the engine are an optional, off-by-default experiment, and never on a command line.
10. The owner's open decisions are Q-01 to Q-09 in `03-integration-plan.md` section 2. Their current answers are in AssistantCabinetAI's `docs/DECISIONS.md` once SP-1 has run.

## 3. Reference documents (absolute paths; read from there, do not copy them)

| What | Path |
|---|---|
| The plan (phases, decisions, budgets) | `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\03-integration-plan.md` |
| The analysis (facts F1 to F20, reuse map, architecture, invariants SI1 to SI12) | `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\02-needs-and-structure-analysis.md` |
| The owner's draft and its review | `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\integration\01-owner-draft-and-review.md` |
| SpeechLab's evidence (numbers, licences, claims table) | `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\SPEECH_ENGINE_EVALUATION.md` |
| SpeechLab licences of voices; listening notes | `...\SpeechLab\docs\TTS_LICENSES.md`, `...\SpeechLab\docs\TTS_LISTENING_NOTES.md` |
| SpeechLab source to port from | `...\SpeechLab\src-tauri\src\speech\`, `...\SpeechLab\src\audio\`, `...\SpeechLab\src\components\ReadAlong.tsx` |
| AssistantCabinetAI | `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI` (remote `https://github.com/dekpo/AssistantCabinetAI`) |
| KB master and workflow (the model for this programme's ritual) | `...\AssistantCabinetAI\docs\SESSION-KB-00-master.md`, `SESSION-KB-WORKFLOW.md`, `SESSION-KB-STATUS.md` |

SpeechLab files are **read-only** in every phase except SP-0. In the speech phases other than SP-0 you work in AssistantCabinetAI only.

## 4. Invariants (each gets a named automated test; the tests are part of the phase that introduces the behaviour)

| ID | Invariant |
|---|---|
| SI1 | Audio is never retained: temp files removed after success, error, cancel, and by a startup sweep; no audio in settings, index, logs or any clip store |
| SI2 | No network from the speech module; no HTTP client in it; the only network use, if any, is an explicit, user-started model installer separate from inference |
| SI3 | A transcript only fills the composer draft; nothing is sent without the user's explicit send |
| SI4 | No speech path reaches apply, fill plans, renames or any outward action |
| SI5 | Dictation never writes to the KB (extends KB I3) |
| SI6 | No transcript, hint or KB text in any log or timing line: counts and milliseconds only |
| SI7 | Read-aloud uses only local voices or engines; a non-local system voice is never selected and never a fallback |
| SI8 | Spoken text is derived by code from the displayed answer; the model never produces it; the speech module adds no model-facing string |
| SI9 | Any speech failure degrades to typed chat with a machine code; speech never blocks launch |
| SI10 | `src-tauri/src/speech/` is ASCII, holds none of the guard's French marker words, has no `#[cfg(windows)]`; every user-visible sentence is an i18n key with fr/en parity |
| SI11 | Cancel works at every stage: recording, transcribing (child killed, temp audio removed), synthesising, playing |
| SI12 | The KB hint path is read-only and selection-scoped; with KB mode `off` it is empty |

KB invariants that bind speech: **I3** (the KB is built only from analysed files and manual edits, never from voice transcripts),
**I7** (entities outside the selection are never revealed), **I8** (nothing the KB stores is sent to a model or logged as text),
**I9** (model-facing strings are profession-neutral), **I10** (Rust returns machine codes), **I13** (no `cfg`, no platform-specific crate).

## 5. AssistantCabinetAI rules you must obey (they override your defaults)

- Read `AGENTS.md`, `CONTRIBUTING.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/LANGUAGE-AND-LOCALE.md`, `docs/PRIVACY-AND-SECURITY.md` before touching code. Check `docs/DECISIONS.md` before proposing something that may be settled.
- **Language contract**: Rust returns machine codes, never prose; every sentence a user reads is a key in `src/locales/{fr-FR,en-US}.json` with parity; a French literal under `src-tauri` is a bug. `src/guards/sources.test.ts` scans every `.rs` under `src-tauri/src` (test modules included) and fails on non-ASCII characters and on the words `le la les des une est erreur fichier dossier envoi veuillez aucune`. Words for numbers, units and abbreviations belong in JSON packs under `src-tauri/resources/`, not in Rust. Integration tests under `src-tauri/tests/` are not scanned.
- Model-facing strings are profession-neutral. Speech adds none.
- UI is TypeScript, never Rust. The webview never talks to the server; everything is a Tauri command.
- No cloud LLM, no cloud speech, no telemetry, no real patient data; fixtures are fictional (`fixtures/gp-sandbox/`, the KB sandbox).
- Settings are `serde(default)`, clamped, with named default constants. Nothing hard-coded that is policy.
- **Git is owner-only.** Never run `git commit`, `git push`, `git merge`, `git rebase`, `git tag`, or open a PR. You may run read-only git. End each phase with the **full copy-paste command block** (branch from `kb/integration` unless the decision Q-03 says otherwise, `git add` of explicit paths, a single-line Conventional Commit subject of at most 72 characters, no body, push, `gh pr create` into `kb/integration`). **No AI attribution anywhere git-facing**: no mention of an assistant, an agent, a model name, a co-author or a "generated by" line, in commits, branch names or PR text.
- Do not install software or change the system without the owner's approval; propose the command.

## 6. The working tree is shared with other tabs

The KB programme runs in other tabs on the same clone. Before you start: `git status` and `git branch --show-current`. If the
tree holds uncommitted work that is not yours, **do not stash, reset, checkout over it or edit those files**; tell the owner and
wait or work in a way that touches only your own new files. Speech code goes in **new files**; shared files
(`commands.rs`, `lib.rs`, `error.rs`, `settings.rs`, `src/lib/ipc.ts`, `SettingsDialog.tsx`, the two catalogues) get the smallest
registration edits. The KB notes warn that lots 7 to 10 touch the same files.

## 7. Technical traps already met (save yourself the rediscovery)

- A shell heredoc through the agent tool **eats backslashes** and rejects long inline scripts with apostrophes: write files with the file tool; in a script use `chr(92)` for a backslash. The Windows working tree is CRLF while the repository stores LF.
- `tauri-build` validates every declared `externalBin` and resource path on **every** `cargo build/test`. Do not declare a sidecar in `tauri.conf.json` unless CI fetches it on both platforms (AssistantCabinetAI `binaries/README.md`). The platform-overlay idea (`tauri.windows.conf.json`) is NOT VERIFIED: spike it before relying on it.
- A sidecar's DLLs must sit next to the sidecar executable. whisper.cpp built as shared libraries needs four DLLs and the Visual C++ runtime (SpeechLab I-070); a static build is cleaner.
- `whisper-cli --prompt` is a command-line argument (no prompt-file option): names in a hint list would be visible in the process list.
- The WebView microphone permission is per origin (`localhost` in development, `tauri.localhost` in the release build). The AssistantCabinetAI CSP has no `blob:`: an AudioWorklet loaded from a blob URL is blocked; ship the worklet as a static file.
- On Windows, `speechSynthesis.getVoices()` includes online voices that send text to a third party: filter on `localService === true` and refuse rather than fall back.
- A dev server of the owner's other work may hold port 1420. Check `tasklist` and the listening ports; never stop processes you did not start.
- Speed figures vary by 10 to 20 % between runs of the same code; quote them with the background load.

## 8. The ritual of every phase

1. Read this file, the launcher, the status file `docs/SESSION-SPEECH-STATUS.md` (AssistantCabinetAI, local; created in SP-1), then the documents the launcher lists.
2. Check preconditions. Create or confirm the branch named in the launcher.
3. Work in small tested steps. Write failures and surprises down as you go.
4. Finish green: `cargo test` (all suites), `pnpm run test`, `pnpm run build` (`build` runs `tsc --noEmit`). A phase that breaks an existing test is not done: fix the cause, never the test, unless the test encoded behaviour the phase deliberately changes (say so).
5. Write the **phase report** `docs/test-reports/speech-programme/lots/sp-<n>-<slug>.md` (what changed by file, what was measured, what was not done, open questions) and, whenever something is observable, a **human test protocol** `docs/test-reports/speech-programme/human-tests/sp-<n>-<slug>.md` for the owner. Always offer at least a smoke test.
6. Update `docs/SESSION-SPEECH-STATUS.md` and the documents the launcher names.
7. Mark every claim **VERIFIED** (observed, with evidence) or **NOT VERIFIED** (with the steps). No invented numbers.
8. Give the owner the git block. Then the final message, in French: what was done, what is verified, what is not, limits, the next launcher to drag.

## 9. Things not to do

Copy SpeechLab wholesale; link a speech engine into the main executable; add a cloud call; auto-send a transcript; write
transcripts, hints or audio to a log, the index or the KB; add a French word to Rust source; add a `cfg`; change KB design or
invariants (propose it to the owner instead); claim macOS support or accent coverage; start the next phase.
