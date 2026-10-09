# Knowledge Base prerequisites for the speech integration

Written 2026-10-09 (status: see `STATUS.md`, ON HOLD). Purpose: tell the Knowledge Base (KB) programme in AssistantCabinetAI what
the future speech module will need from it, say what must be true before the integration plan is re-baselined and resumed, and
list the facts to re-verify. The KB programme is the owner's current work; **nothing here asks the KB to build speech, and nothing
here changes the KB design**: where something below would, it is a question for the owner.

Sources of the KB side: `AssistantCabinetAI/docs/SESSION-KB-00-master.md` (section 13 "Future contracts"), `SESSION-KB-01-foundation.md`
(lots 2, 3), `SESSION-KB-04-surfaces.md` (lot 9), `docs/DECISIONS.md` ("Knowledge Base and the product direction", D1 to D10).

## A. What speech will need from the KB (contracts to keep stable)

The KB master already promises most of these. They are listed so that a lot report mentions any change.

| # | Contract | Used by | Notes |
|---|---|---|---|
| P-1 | `surface_forms(selection, limit, purpose = SpeechHints)` returning `{entity_id, display, normalized, phonetic_key, spoken_form, type_id, subtype, weight}`, ranked, **scoped to the selection**, callable from a Tauri command without the gateway and without a UI | dictation hints; spoken forms for read-aloud | must be fast (milliseconds) and must log counts only (I8). Speech never needs entity ids to leave the process |
| P-2 | `repair::propose(transcript, selection, locale, case_reliable)` returning spans with candidates and `Suggest`/`Ask`/`None`; **pure text in, proposals out, never rewrites, never writes** | repair-first spelling after dictation | engine-independent; selection-first (I7) |
| P-3 | Contract fixture `tests/fixtures/knowledge/repair-fr.json` with `heard -> expected` pairs **and must-not-change pairs** | regression set shared with speech | the speech side will contribute real recogniser errors (SpeechLab `repair-pairs-candidate.json`, produced by SP-0) |
| P-4 | `spoken_form` attribute (manual, editable in the management dialog of lot 8) and the pack table `title_spoken` | read-aloud pronunciation of names; titles ("dr" to "docteur") | the owner will want to correct how a name is pronounced; make sure lot 8 exposes `spoken_form` |
| P-5 | Pack loader and JSON pack format (`resources/knowledge/...`, lot 3) | the speech rule packs (`resources/speech/rules/{fr-FR,en-US}.json`: number words, units, months, abbreviations) should follow the same conventions and, ideally, reuse the loader | any pack-format decision made in lot 3 is relevant: report it |
| P-6 | A KB read entry point that honours `knowledge_mode` (`off` gives nothing) | `SpeechHintSource` adapter | I4: failure or empty means fall back, never an error shown to the user |
| P-7 | A French phonetic key that is versioned (`kb_meta.phonetic_version`) | repair candidates | question for the owner: dictation in English (`en-US`) has no phonetic encoder in the plan; is English repair wanted? |
| P-8 | Names hidden on screen by default (`knowledge_hide_names_in_suggestions`, D10) | repair chips in the composer | the speech UI must follow the same setting |
| P-9 | Entity `item` with `subtype = medication` from the health pack, and a decision on **how drug names enter the KB** (only from the user's analysed files, per I3, or also from a shipped list) | medicines are a core speech need (every engine misspelled drug names in SpeechLab; best 4 of 6) | owner's decision; SpeechLab warns that a shipped drug list is clinical content to maintain (D-036) and that a dictionary cannot tell a misspelling from a rare real word (I-045) |

## B. Conditions to resume (the re-baseline gate)

The owner decides when to resume; these are the points the re-baseline pass will check. Marking them is the owner's.

| # | Condition | How the pass verifies it |
|---|---|---|
| G-1 | The owner states that the KB implementation is finished (or finished enough), and which lots are merged | the owner's message; `docs/SESSION-KB-STATUS.md`; `git log` and tags `kb-after-lot-NN` |
| G-2 | KB lots **2, 3, 6, 9** are merged into the integration branch, and ideally lot 8 (management dialog, `spoken_form`) and lot 11 (release gate) | read `knowledge/{normalize,phonetic,packs,surface,repair}.rs` and their tests; do not trust this list |
| G-3 | Whether `kb/integration` has been merged into `main`, so that the base branch of the speech work (Q-03) is known | `git ls-remote --heads`, `git log main` |
| G-4 | The owner has used the KB on fictional data and can say what is missing for names, organisations and medicines | the owner's notes (the pass will ask) |
| G-5 | The macOS situation (still no Mac?) and whether AssistantCabinetAI's "both platforms mandatory" rule changed | `AGENTS.md`, the owner |
| G-6 | Optional: the practice-PC specification and, if SP-0 was run, its probe results | `docs/integration/PROBE_RESULTS.md`, the owner |
| G-7 | Any new decision in AssistantCabinetAI's `docs/DECISIONS.md` about speech, models, licences or the client since 2026-10-09 | `git log -p -- docs/DECISIONS.md` since `d215939`, and a read of the new sections |

## C. What KB lots should not break (coordination, not a request to build)

These are things the speech module will rely on or edit. If a lot has to change one of them, say so in its report under
"Speech-relevant changes".

1. `Composer.tsx` receives `draft` and `onDraftChange` from the chat panel; dictation will insert text through them. Changing who owns the draft changes the integration.
2. The per-answer action row in `MessageList.tsx` (copy, edit, regenerate) is where read-aloud will be added.
3. `settings.rs` conventions (`serde(default)`, clamp on the way out, named constants) and the "Advanced" group of `SettingsDialog.tsx`.
4. The language guard `src/guards/sources.test.ts` (no non-ASCII and no French marker words in `.rs` under `src-tauri/src`). The speech rule packs depend on data files being allowed.
5. The catalogue parity test, and the machine-code error pattern in `error.rs`.
6. `cancellation.rs` (one run in flight, drop the future).
7. `sidecar_search_roots` in `commands.rs` and the rule that no sidecar is declared in `tauri.conf.json` until CI fetches it on both platforms.
8. The content security policy in `tauri.conf.json` (no `blob:` today) and `capabilities/default.json`.
9. The CI matrix (`windows-latest` and `macos-latest`, `cargo test --lib`).
10. Invariants I3, I7, I8, I9, I10, I13 (they bind speech too).

## D. Facts to re-verify at the re-baseline (starting commands)

Run from `C:\Users\elise\Documents\CURSOR\AssistantCabinetAI`, read-only.

```text
git fetch --dry-run ; git ls-remote --heads origin ; git branch -a --sort=-committerdate ; git tag --list "kb-*" ; git log --oneline -20 main kb/integration
git log -p --follow -- docs/DECISIONS.md   (new sections since d215939)
read docs/SESSION-KB-STATUS.md, docs/KNOWLEDGE-BASE.md (if it exists), docs/test-reports/knowledge-base-pass-1/
read AGENTS.md and .cursor/rules/*.mdc  (macOS rule, voice rules, language rules, git rules)
read apps/desktop/src-tauri/Cargo.toml, tauri.conf.json, capabilities/*.json, .github/workflows/ci.yml, binaries/README.md
read apps/desktop/src-tauri/src/knowledge/{mod,normalize,phonetic,packs,surface,repair,guard}.rs   (signatures and tests)
read apps/desktop/src/components/{Composer,ChatPanel,MessageList,SettingsDialog}.tsx, src/state/useChat.ts, src/lib/ipc.ts
count lines of commands.rs, ipc.ts, locales (merge-conflict surface)
```

The facts to re-check are F1 to F20 of `02-needs-and-structure-analysis.md`; the re-baseline report records for each one
"unchanged", "changed (what)" or "obsolete".

## E. What the owner can note while finishing the KB (inputs for the re-baseline)

Not homework; a list of things that will make the next pass better. Write them anywhere; the pass will ask.

1. Names, organisations and medicines you most want recognised and spelled well (a dozen real-looking fictional examples).
2. Names that are hard to pronounce when read aloud (to test spoken forms).
3. Whether English matters for dictation or only French.
4. Whether the KB, as built, can hold medicines and how you intend to feed it.
5. Anything in the KB's interface that changes where a microphone button or a read-aloud button would feel natural.
6. A Mac, if one becomes available; the practice PC's model, CPU, memory and Windows version.
7. Any decision from the plan's list Q-01 to Q-09 that you already know the answer to.
