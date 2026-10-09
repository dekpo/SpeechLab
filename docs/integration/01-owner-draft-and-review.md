# The owner's statements and draft integration plan, with the review

> **STATUS: ON HOLD (D-050), draft v1 of 2026-10-09.** The owner will finish the Knowledge Base first and then relaunch a pass on
> the plan to adapt it. See `STATUS.md`. The review below reflects AssistantCabinetAI as read on 2026-10-09 and must be re-checked by
> the re-baseline pass (`docs/prompts/integration/SP-R-rebaseline.md`).

Recorded 2026-10-09. The owner speaks French; the repository is English. The draft is kept **verbatim in French** (it is the
owner's text, as for the listening notes), followed by an English gloss and the review. The review compares the draft with
the evidence of SpeechLab (`docs/SPEECH_ENGINE_EVALUATION.md`) and with the state of AssistantCabinetAI read on the same day
(`02-needs-and-structure-analysis.md`, facts F1 to F20).

## 1. Information given by the owner on 2026-10-09 (recorded as decisions, D-048)

1. **No Mac is available** to implement and test a macOS version. **The macOS part waits until further notice.** It is not
   cancelled: `docs/MACOS_VALIDATION.md` stays the checklist, every line still NOT VERIFIED.
2. **The work on SpeechLab is considered finished.** The next work is the integration of speech-to-text (STT) and
   text-to-speech (TTS) into AssistantCabinetAI. SpeechLab is frozen as the reference and the measurement bench (D-048).
3. AssistantCabinetAI lives at `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI` and on GitHub at
   `https://github.com/dekpo/AssistantCabinetAI`. Neither is finished: a **Knowledge Base (KB)** is being implemented to
   structure the user's data, create entities and types, and let the STT and TTS tools use that knowledge for better
   understanding, spelling and pronunciation of **proper names, organisations and medicines**.
4. The work of preparing the integration is **not to be started in the chat where it was requested**: the information is
   documented first, and one or several prompt files are produced for use in other chat tabs, so that the owner can submit
   the plan step by step.
5. The owner supplied a **draft integration plan** to be studied in detail, compared and corrected (section 2 and 3 below).
6. Do not copy all of SpeechLab into AssistantCabinetAI: reuse the necessary components behind a stable interface, with
   isolated dependencies and a configuration that can be switched on and off.

## 2. The owner's draft, verbatim

> Je te conseille d'intégrer SpeechLab dans AssistantCabinetAI progressivement, comme un module autonome interchangeable, sans mélanger la reconnaissance vocale avec le cœur conversationnel de l'application.
>
> ## Plan d'intégration proposé
>
> Étape 1 — Audit et gel de SpeechLab
>
> Prérequis
>
> Valider les moteurs retenus, les langues, la précision du français, les performances et les tests. Documenter les dépendances et les contraintes Windows (et le prévoir pour macOS).
>
> Étape 2 — Créer une interface SpeechProvider
>
> Définir un contrat indépendant du moteur : initialisation, disponibilité, démarrage/arrêt de l'écoute, transcription, erreurs et libération des ressources. Prévoir plusieurs adaptateurs interchangeables.
>
> Étape 3 — Intégrer la dictée dans l'interface
>
> Ajouter un bouton microphone dans le chat. La transcription remplit le champ de saisie, reste modifiable, puis l'utilisateur l'envoie explicitement.
>
> Étape 4 — Configuration et confidentialité
>
> Permettre de choisir le moteur, la langue et les options vocales. Privilégier le traitement local adsolument, sans transmettre les enregistrements à un service externe ni sur le réseau local.
>
> Étape 5 — Synthèse vocale (TTS)
>
> Ajouter ensuite un bouton pour écouter la réponse de l'assistant, avec un moteur indépendant de celui de la transcription. Permettre un mode de conversation automatique activable par l'utilisateur dans les réglages du frontend pour qu'il puisse converser à haute voix avec le modèle choisi.
>
> Étape 6 — Tests et validation
>
> Vérifier la précision, les accents français et anglais, les termes médicaux, les interruptions, les permissions microphone et le fonctionnement hors ligne sur Windows (et le prévoir pour macOS).
>
> ## Ordre de travail recommandé
>
> 1. Examiner la branche `main` d'AssistantCabinetAI et surtout les nouvelles branches actuelles qui concerne l'implémentation d'une base de connaissance Knowledge Base et identifier les points d'intégration dans le chat, la configuration et le cycle de vie de l'application.
> 2. Définir le contrat `SpeechProvider` et son adaptateur pour le moteur retenu dans SpeechLab.
> 3. Implémenter uniquement la dictée vocale, puis tester.
> 4. Ajouter les paramètres et la gestion des erreurs.
> 5. Intégrer le TTS dans un second sprint.
>
> Point important : ne surtout pas tout copier de SpeechLab dans AssistantCabinetAI. Réutilise ses composants nécessaires derrière une interface stable, avec des dépendances isolées et une configuration activable/désactivable.

(Typos left as written: "adsolument", "sourtout" in the message, "qui concerne".)

English gloss: integrate progressively as an autonomous, interchangeable module, never mixing speech recognition with the
conversational core. Six steps: (1) audit and freeze SpeechLab; (2) a `SpeechProvider` interface (initialise, availability,
start/stop listening, transcribe, errors, release resources; several adapters); (3) a microphone button in the chat, the
transcript fills the editable input and is sent explicitly; (4) configuration and privacy (engine, language, voice options;
local processing only, no recording sent to an external service or over the local network); (5) TTS later, with an engine
independent of the transcription engine, and an automatic conversation mode the user enables in the front-end settings;
(6) tests (accuracy, French and English accents, medical terms, interruptions, microphone permission, offline, Windows and,
planned, macOS). Order: examine `main` and the new KB branches; define the contract and an adapter; dictation only, then
test; settings and errors; TTS in a second sprint. Do not copy everything from SpeechLab.

## 3. Review: what to keep, what to change, what is missing

Verdicts: **Keep**, **Keep with change**, **Correct** (the draft would cause a problem), **Add** (missing).

### 3.1 The principle

| Draft | Verdict | Review |
|---|---|---|
| Autonomous, interchangeable module, separate from the conversational core | **Keep** | Fully consistent with the AssistantCabinetAI architecture (ports behind interfaces, `OcrProvider` precedent, F1) and with the KB's own treatment of speech as "contracts only" (master section 13). Sharpen it into a rule: the speech module's only links to the product are text in (composer draft, answer text), text out (the transcript) and **read-only** KB hints; switching it off or deleting it changes nothing else |
| "Progressively" | **Keep** | Matches the phase order of the plan (dictation, then settings and errors, then KB spelling, then read-aloud, conversation last) |

### 3.2 Step by step

| Step | Verdict | Review and suggested wording |
|---|---|---|
| **1. Audit and freeze** | **Keep with change** | The audit **exists** (`SPEECH_ENGINE_EVALUATION.md`: engines, languages, French accuracy, performance, tests, dependencies, Windows constraints). What the step still needs: (a) the freeze made explicit (D-048, an owner-run git tag, a status banner), (b) the **exit criteria that the audit could not provide**: a measurement on the **actual 2019 practice PC** (SpeechLab's numbers are from a 2024 laptop), and the **licence decisions** (CC BY 4.0, GPL phonemizer). Accuracy figures concern **one speaker**; "accents" were dropped (D-046), so "French precision" must be stated as such. macOS: "plan for it" becomes "on hold, with a recorded waiver" (Q-04) |
| **2. `SpeechProvider` interface** | **Correct** | (a) **Two ports, not one**: `SpeechToTextProvider` and `TextToSpeechProvider`, otherwise step 5's "independent engine" is violated by construction. (b) **"Start/stop listening" is not a provider job**: microphone capture is webview code (`getUserMedia`, AudioWorklet), as in SpeechLab D-020 and AssistantCabinetAI "UI in TypeScript, never in Rust"; the Rust provider receives a finished clip and returns text. (c) Streaming/partial results were **never evaluated**; the contract is whole-clip. (d) Add what the contract really needs: **availability states** (engine missing, model missing, locale unsupported) as machine codes, **cancellation** (real, by killing the child), **hints** (optional vocabulary), and a **read-only hint source** for the KB. (e) Engines run **out of process** (GPL isolation, 1.9 GB, kill-to-cancel). (f) Follow the `OcrProvider` pattern: id, version, one call per unit of work, errors as codes, degrade-to-unavailable |
| **3. Dictation in the interface** | **Keep with change** | The rule (fills the box, editable, explicit send) is exactly right and becomes invariant SI3. Add: insertion at the cursor, one-step undo, a level/clipping warning (built in SpeechLab), explicit states and cancel at each state, disabled-with-explanation when the engine or model is missing, keyboard access and a live status for accessibility, and the **CSP** change needed for the microphone worklet (AssistantCabinetAI's policy has no `blob:`). The wait is real: whisper.cpp small ran slower than real time on SpeechLab's laptop |
| **4. Configuration and privacy** | **Keep with change** | "Local only, nothing to an external service nor on the local network" agrees with KB decision D7 and with the voice rules in `DECISIONS.md`; the reserved `/v1/audio/*` server routes (LAN) should be **recorded as not used**. Add: audio is never retained (temp file lifecycle, SI1), no text in logs (SI6), speech code has no HTTP client (SI2), an offline proof at the release gate, and **how the model files reach a machine without mandatory Internet** (Q-05). Settings follow the existing pattern (named default constants, `serde(default)`, "Advanced" group). The language is not a free choice: it follows the interface `locale` (fr-FR, en-US) unless overridden |
| **5. TTS** | **Keep with change** | "A button to listen to the answer" and "an engine independent of the transcription engine": keep. Changes: (a) only **local** voices (the Windows system list includes online voices that send text to a third party; filter and refuse, with a test); (b) first path = **system voices** (no sidecar, no GPL, both platforms), Piper only behind the same port and only if the owner accepts the licensing conditions (section 7 of the analysis); (c) the **speakable text** must be derived by code from the answer's Markdown (citations and links removed, numbers and units spoken), never by the model; (d) names are pronounced through KB `spoken_form`. (e) **The automatic conversation mode contradicts step 3 and the project rules**: it implies sending dictated text without an explicit send. See 3.3 |
| **6. Tests and validation** | **Keep with change** | Keep: precision, medical terms, interruptions, microphone permission, offline on Windows. Change: "French and English **accents**" cannot be claimed (no accent data exists; dropped by the owner): replace by **the pilot GP's own voice and microphone** plus a small speaker panel, with intervals and the number of speakers stated. Add: performance budgets on the practice PC, named invariants SI1 to SI12, CSP and packaging checks, the KB invariants (I3, I8), the source-language guard, catalogue parity. macOS: on hold |

### 3.3 The automatic conversation mode (draft step 5)

The draft asks for a mode where the user "converses aloud with the chosen model". That is feasible but it is **a different
feature from dictation**, and it conflicts with rules already written:

- Step 3 and the project rule say the user sends **explicitly**; a hands-free loop (listen, transcribe, send, answer, read,
  listen) removes that act. Dictated text can contain a wrong number or name; nothing re-reads it. SpeechLab's own rule is
  "do not auto-send transcripts to an AI agent" (`Plan.md` section 15).
- The existing frozen voice rules say a voice command must never trigger `apply` on files; a conversation loop must provably
  never reach file actions, fill plans or any outward action (SI4).
- It needs engineering that dictation does not: end-of-speech detection (the Silero VAD exists in SpeechLab), a visible
  confirmation or countdown before sending, **barge-in** (interrupting the voice), **echo control** (SpeechLab's recorder
  turned echo cancellation **off** on purpose; with speakers playing the answer the microphone will hear it), and CPU headroom
  for recognition and synthesis together on a 2019 PC.
- KB decision D8 allows "read answers automatically" only as "a possible later addition" and keeps read-aloud off by default.
  Reading patient-related answers aloud in a practice can be overheard (the same concern as KB decision D10 on screen).

Proposal: keep the conversation mode as the **last, optional phase (SP-7)**, behind its own decision (Q-07), with push-to-talk
first, an explicit confirmation step before send, and a hard separation from file actions.

### 3.4 The recommended work order

| Draft item | Verdict | Review |
|---|---|---|
| 1. Examine `main` and the KB branches | **Correct** | `main` is **deliberately frozen** until the KB release gate (lot 11); the live base is `kb/integration` (`d215939`), with lot 1 in progress on `feat/kb-store`. This examination is **done** (analysis, section 2). The base-branch choice is decision Q-03 (recommended: `kb/integration`) |
| 2. Define the contract and an adapter | **Keep with change** | Two ports, plus the fake provider for tests; first adapter = **whisper.cpp sidecar** (MIT, existing sidecar pattern, KB decision D7), second = a worker hosting sherpa-onnx (Parakeet) only if the owner accepts CC BY 4.0 and the practice-PC probe needs it. This refines SpeechLab's D-047 (Parakeet default) for the **integration order**; it does not contradict the evidence |
| 3. Dictation only, then test | **Keep** | Phases SP-2 to SP-4 |
| 4. Settings and error handling | **Keep with change** | Done together with model availability (SP-3) so the interface never offers a microphone that cannot work |
| 5. TTS in a second sprint | **Keep** | SP-6, after the KB-assisted spelling (SP-5) because spoken forms come from the same KB lots |

### 3.5 "Do not copy everything"

**Keep** and make operational with the reuse map (analysis section 4): port the engine-facing ideas and the webview capture
code, rewrite the French/English text rules as data packs (the source-language guard forbids them as Rust source), leave all
evaluation tooling in SpeechLab, and never link a speech engine into the main executable.

### 3.6 Missing from the draft

1. **The Knowledge Base**, the owner's main concern: repair-first spelling, spoken forms, hint lists, selection scope,
   invariants I3/I7/I8, and the dependency on KB lots 2, 3, 6 and 9 (analysis section 8).
2. **The hardware reality**: a 2019 Windows PC of unknown specification. No engine is chosen before the probe.
3. **Licence policy**: CC BY 4.0 needs an explicit decision (KB D7); the GPL phonemizer; the French voice lineage; credits.
4. **Model distribution** without mandatory Internet; the checksum manifest; `models/LICENSES.md` rows.
5. **Packaging traps** already met by OCR: `tauri-build` validates sidecar paths on every build; a platform-specific config
   overlay may solve it while macOS is on hold (to spike).
6. **CSP and capabilities** for microphone and playback.
7. **Language contract and source guard** (no French in Rust source, data packs, catalogue parity).
8. **Sequencing against the KB lots** and the merge conflicts on shared files.
9. **Privacy details**: hints on a command line, temp audio, read-aloud overheard, transcripts never in logs or the KB.
10. **Rollback**: a setting that turns speech off and a module whose removal leaves the product intact.
11. **Accessibility** of the new controls, and **error taxonomy** as machine codes.

### 3.7 Summary of corrections to the draft

1. Two ports instead of one `SpeechProvider`; capture stays in the webview; contract is whole-clip.
2. Engines out of process; first adapter whisper.cpp sidecar; Parakeet only after an explicit licence decision.
3. Probe the practice PC before choosing; the audit alone cannot decide.
4. Base branch is `kb/integration`, not `main`.
5. Replace "accents" by the real users' voices; no accent claim.
6. Conversation mode last and separate; it must not weaken explicit send.
7. TTS starts with local system voices; Piper only behind the licensing decisions.
8. Add the KB integration as its own phase.
9. macOS: on hold, with a recorded waiver of the "both mandatory" rule.
