# HANDOFF — read this right after `AGENTS.md`

> **STATUS 2026-10-09: SpeechLab is FINISHED and FROZEN (D-048).** M0 to M8 are delivered. It is the reference (evidence in
> `docs/SPEECH_ENGINE_EVALUATION.md`) and the measurement bench; it is reopened only to measure or for phase SP-0. **macOS is ON
> HOLD until further notice** (no Mac, I-075). The next work is the integration of speech into AssistantCabinetAI:
> `docs/integration/` (analysis, review of the owner's draft, plan) and `docs/prompts/integration/` (master + one launcher per
> phase, for other chat tabs). Never modify the AssistantCabinetAI repository from here.
>
> **THE INTEGRATION IS ON HOLD (D-050, 2026-10-09): the owner finishes the Knowledge Base in AssistantCabinetAI first, then a
> re-baseline pass adapts the plan.** Read `docs/integration/STATUS.md`. Do not run any `SP-*` launcher until it says `STATUS:
> RESUMED` (the pass is `docs/prompts/integration/SP-R-rebaseline.md`; SP-0 only if the owner explicitly asks). The documents in
> `docs/integration/` are draft v1 and their facts about AssistantCabinetAI must be re-verified. Prerequisites for the KB side:
> `docs/integration/KB-PREREQUISITES.md`; note for KB tabs: `docs/prompts/integration/KB-TAB-NOTE.md`.

For any agent or person starting a new session. It tells you where the project stands, what the
traps are, how to run things, and what to do next. Keep it up to date: when you finish a step,
change the "Where things stand" and "Backlog" sections in the same commit as your work.
All repository text is English. The owner reads and writes French: answer the owner in French in
the chat, write files in English.

## 1. The project in five lines

- **AssistantCabinetAI-SpeechLab**: an experimental, offline lab that evaluates open-source
  speech-to-text (whisper.cpp, sherpa-onnx) and text-to-speech engines, to decide what to
  integrate later into AssistantCabinetAI (a separate product, never touched from here).
- Desktop app: Tauri 2 + React + TypeScript + Rust. Engines run locally, CPU only on the dev machine.
- Mission and constraints: `Plan.md`. Rules for everyone: `AGENTS.md`. Decisions: `docs/DECISIONS.md`.
  Problems: `docs/ISSUES.md`. Chronological journal: `docs/PROJECT_LOG.md` (read its last entries).
- Goal: measured, reproducible evidence (accuracy, critical errors, speed, memory, licences), not
  opinions. Nothing simulated, nothing extrapolated, every claim marked verified or not verified.
- Final deliverable: `docs/SPEECH_ENGINE_EVALUATION.md` (milestone M8).

## 2. Rules that get broken most often (full text in `AGENTS.md`)

1. **Never run `git commit`, `git push`, `git merge`, `git tag` or open a PR.** Give the owner the exact commands instead (branch, add, one-sentence English commit, push). Read-only git is fine.
2. **Never mention any AI assistant, model, vendor or related URL** anywhere in the repository (code, docs, commits, file names, config). No "Co-Authored-By", no "generated with".
3. **Write down everything in `docs/`**: successes, failures, dead ends, surprises, in the log. Mark each claim VERIFIED (observed here, with evidence) or NOT VERIFIED (with the steps to verify).
4. **Free and open-source first**; reject non-commercial or copyleft models/libraries without the owner's approval (known flag: espeak-ng is GPL-3.0, see D-012).
5. **Do not install software or change the system without the owner's approval.** Propose the command.
6. **No private content in the repository or in documents**: voices of third parties, the owner's voice notes, the first name of the addressee of a private message, personal data. Only aggregate numbers. Trap (I-047): `summary.md` and `runs.jsonl` of a private run (`benchmark/results/private/`) contain the clips' references and transcripts; never paste from them, always pass `--category` with `--include-private`.

## 3. Where things stand

| Milestone | Status |
|---|---|
| M0 feasibility | done, merged |
| M1 Tauri skeleton and provider contract | done, merged |
| M2 sherpa-onnx STT (model manager, downloader, UI) | done, merged |
| M3 whisper.cpp STT (external `whisper-cli` process, real cancellation) | done, merged |
| M4 microphone, audio import, clip store, engine comparison view, WER/CER | done, committed |
| M5a dataset format, 95 reading scripts, number folding, critical-error detector | done, pushed |
| M5b in-app dataset recorder | done |
| M5c benchmark runner, full benchmark | done: first run (disturbed) plus clean re-run `20261007-215116-full-owner-reps1-clean` (855/855 transcripts identical; speed figures measured at about 20 % background CPU, D-034). Variance still unmeasured (1 repetition) |
| M5d timing study (3 repetitions) | done: `20261008-035938-timing-3reps`, 40 short sentences x 9 configurations x 3 repetitions, 0 changing transcripts, within-run spread 1 to 4 % (I-012 partly closed) |
| M5e long audio, chunking at silences (Silero VAD) | done: D-035; 828/828 short transcripts unchanged with chunking on; helps sherpa Whisper tiny (30 s limit), neutral for Parakeet and whisper.cpp, harmful for Canary (I-041); Whisper loops not fixed |
| T4 drug names / key terms (D-036) | done: strict post-correction 0 broken words and fixes 2 of 6 drug names for the best engines; whisper.cpp prompt helps small/base but costs speed; Parakeet hotwords need a surrogate vocabulary and over-boost at score 3.0; all OFF by default; upper bound (vocabulary taken from the test sentences) |
| M6 TTS laboratory (D-038, D-039) | built and measured 2026-10-08: provider, 6 voice packages (Piper siwis, gilles, libritts_r, ljspeech; Coqui css10 fr; Kokoro v1.0 int8), UI tab, `tts` CLI, runs `20261008-085403-tts-m6` and `20261008-094413-tts-m6-licence-focus` (8 voices in one run). **Licence-first (owner's rule): every voice has a commercial-use rating, see `docs/TTS_LICENSES.md`**; only `clear`/`attribution` voices may be recommended. Piper and Coqui are 13 to 29 times faster than real time, Kokoro int8 slower than real time (RTF 1.1 to 2.1 depending on the run). **Owner's blind listening DONE** (`docs/TTS_LISTENING_NOTES.md`): chose Piper siwis, gilles, ljspeech, libritts_r; English clean voices are excellent, the free French voice (Coqui) is weak and does not speak digits, Kokoro has an audible hiss (I-057). Owner's decisions: D-040 accepted (sentence highlight + auto-scroll, no click-to-jump yet), D-041 (beta only with `clear`/`attribution` voices, no paid licence, public domain preferred). `kristin` (public domain) installed. NEXT: `docs/prompts/06b-tts-readalong-and-normaliser.md` |
| M6d TTS read-along, sentence splitter, text normaliser (D-042) | built and tested 2026-10-08: sentence-by-sentence synthesis joined into one WAV with measured segments (250 ms / 600 ms pauses), grey sentence highlight driven by the player's clock with auto-scroll, "Follow reading" and manual-scroll pause, digits-to-words normaliser (French and English) on by default for the Coqui voice through the manifest flag `normalizeText`, preview of the text sent to the voice. 163 Rust tests, 16 vitest, 33/33 UI checks over DevTools (`scripts/ui_test_readalong.mjs`). Overhead of per-sentence synthesis a few percent (run `20261008-141002-tts-m6d-overhead`); `kristin` measured (RTF 0.055 to 0.064); normaliser: key numbers heard 0/36 off, 28/36 and 30/36 on (rough recogniser check). Piper `mls` French voice converted locally (`packaging: local`, I-060) and weak by a recogniser proxy. **Owner's listening session 2 DONE and recorded** (`docs/TTS_LISTENING_NOTES.md`): `mls` rejected (1 to 2), Coqui with the normaliser 3 out of 5, English ljspeech best of the session; the owner's interface tests rank siwis and gilles (French) and libritts_r, kristin, ljspeech (English) as good, with the rewriting on for every voice. **French voices: the owner keeps siwis and gilles for now and will re-train them from the clean base checkpoint before any commercial release (D-043 accepted, I-063 = commercial-release blocker; D-041 amended for these two voices).** Chatterbox and Qwen3-TTS read: not recommended (data licences unstated, slow on CPU). NEXT: M7 |
| M7 Windows/macOS packaging validation (D-044, D-045) | Windows DONE 2026-10-08: per-user NSIS installer (8,858,030 bytes), `whisper-cli.exe` as sidecar + 4 DLLs, no model bundled, strict CSP; installed and tested (33/33 read-along checks, 23/24 packaged-app checks, 11/11 first start without models, microphone prompt and persistence in the release origin, offline proof for the app's own processes: STT with both engines and TTS work, a download is blocked). Open: I-065 (cancel banner), I-066 (web view runtime connects to a Microsoft address), I-067 (no credits view / installer licence page), I-068 (unsigned, SmartScreen NOT VERIFIED), I-069, I-070 (sidecar needs the Visual C++ runtime; clean machine NOT VERIFIED). macOS: checklist only (`docs/MACOS_VALIDATION.md`), NOT VERIFIED. The owner must remove the firewall rule (`scripts\offline_proof_remove.cmd`) |
| M8 final report, licensing table, recommendation (D-047) | DONE 2026-10-09: `docs/SPEECH_ENGINE_EVALUATION.md` (12 sections plus claims table A, reproduction B, corrections C). Every number cross-checked against `summary.md`, `bootstrap_ci.py`, `timing_study.py` and the TTS summaries (six small documentation errors fixed, listed in the report's Appendix C). Recommendation PROVISIONAL: both engines behind the provider contract, Parakeet as default speech-to-text, whisper.cpp small as second provider, Piper English voices for TTS, French TTS and any build that synthesises speech blocked by the voice lineage (D-043) and the GPL phonemizer (95 occurrences of "espeak" in the packaged `speechlab.exe`). New open items I-071 to I-074 (I-071 fixed). No new measurement was made. NEXT: owner's choice, see backlog |
| Freeze and integration preparation (D-048, D-049 proposed) | DONE 2026-10-09 (documents only, nothing built): SpeechLab frozen, macOS on hold (I-075); `docs/integration/` = the owner's draft recorded verbatim and reviewed, analysis of AssistantCabinetAI read-only (facts F1 to F20, e.g. licence policy Apache/MIT only with CC BY by explicit decision, a 2019 practice PC, source-language guard forbidding French words in Rust, CSP without `blob:`, KB branch model on `kb/integration`), reuse map, architecture (two ports, engines out of process, repair-first KB use), invariants SI1 to SI12, plan SP-0 to SP-8 with decisions Q-01 to Q-09; prompts in `docs/prompts/integration/` (I-076). **ON HOLD since the same day (D-050): the owner finishes the Knowledge Base first; then the re-baseline pass (SP-R) adapts the plan.** NEXT: nothing for speech until the owner says the KB is finished; then SP-R |

Check `git log --oneline -5` and `git status` (read-only) to see what the owner has committed.

**What the benchmark says (one speaker, CPU, 1 repetition, speed at about 20 % background CPU)**:
sherpa-onnx Parakeet TDT v3 int8 has the lowest WER (2.3 %) at RTF 0.11, Canary int8 is comparable
on clean input but once produced garbage, whisper.cpp small is accurate (3.9 %) but still slower
than real time on this CPU (RTF 1.1 to 1.3), tiny/base Whisper are too inaccurate. Every engine
misspells drug names. Accuracy is load-independent (clean re-run identical to the disturbed run).
Timing study: inside one run the same sample varies by 1 to 4 % (median) and never changes its transcript; between two runs the fast models differed by up to 20 %, so quote speed with a 10 to 20 % margin. Details and caveats: last M5c and M5d entries of `docs/PROJECT_LOG.md`, results in `benchmark/results/`.
Long audio (M5e, 3 reference dictations + 2 private clips, little statistical power): only Parakeet and whisper.cpp had no failure; sherpa Whisper tiny cannot take more than 30 s and loops; Canary drops endings and degrades on 24 s pieces. Speed figures differ by 1 to 48 % between runs (I-042): compare inside one run only.
Drug names (T4, D-036): every engine misspells them (best: 4 of 6 found). Strict dictionary post-correction fixed 2 more with no word broken; vocabulary biasing inside the engine is riskier (loose correction and high hotword score are regressions). One speaker, 6 drug occurrences, vocabulary taken from the test sentences: an upper bound.
TTS (M6, D-038/D-039): Piper and Coqui voices are interactive on this CPU (RTF 0.03 to 0.08, 210 to 260 MB), Kokoro int8 is not (RTF 1.1 to 2.1 depending on the run, 430 to 460 MB); speed control works on both but differently; the voices are RATED for commercial use (cleanest found: English Piper ljspeech `clear`; French Coqui css10 `attribution`, needs no phonemizer but does not speak digits, I-055); Piper siwis/Kokoro need a legal reading, gilles is excluded (I-052, I-053); any build that synthesises speech links the GPL espeak-ng code (I-051, I-054). Quality of the voices is the owner's to judge.
TTS read-along (M6d, D-042): per-sentence synthesis costs a few percent (within the run-to-run spread), the pauses make the audio 7 to 9 % longer; the Coqui voice speaks numbers once the normaliser rewrites them (0/36 to 28-30/36 key numbers heard by a recogniser) but remains weak otherwise (I-058); the converted Piper `mls` voice is a weak candidate by the same proxy (I-060); Chatterbox and Qwen3-TTS are not usable as free voices on what was read (docs/TTS_LICENSES.md).
Final report: `docs/SPEECH_ENGINE_EVALUATION.md`; recommendation D-047 (provisional; D-032 is superseded by it).

## 4. Repository map

```
Plan.md, AGENTS.md           mission and rules
docs/                        PROJECT_LOG, DECISIONS, ISSUES, GIT_WORKFLOW, HANDOFF, M0 report, SPEECH_ENGINE_EVALUATION (final report), MACOS_VALIDATION, TTS_LICENSES, TTS_LISTENING_NOTES, prompts/
benchmark/scripts/           95 sentences to read aloud (committed)
benchmark/samples/           metadata + reference for each recorded sample (committed, no audio)
benchmark/samples-private/   same, for third-party voices (git-ignored, never quote)
benchmark/audio/             the WAV recordings (git-ignored)
benchmark/results/<run>/     benchmark outputs (summary.md, summary.csv, runs.jsonl, system.json, config.json)
src/                         React UI (components, audio capture, engine-agnostic TypeScript contract)
src-tauri/src/speech/        Rust: provider traits, sherpa.rs, whisper_cpp.rs, models.rs, download.rs, clips.rs,
                             dataset.rs, metrics.rs, numbers.rs, critical.rs, probe.rs, benchmark.rs,
                             chunking.rs + vad.rs (long audio cut at silences),
                             postcorrect.rs (dictionary correction, OFF), termstudy.rs (key-term study),
                             tts.rs (sherpa-onnx text-to-speech provider, voices from the manifest; per-sentence synthesis, segments),
                             sentences.rs (sentence splitter, UTF-16 spans), normalise.rs (digits and units to words, fr/en)
src-tauri/examples/          CLI tools: transcribe.rs (one file), bench.rs (benchmark; `bench import` adds private clips), hotwords_probe.rs (T4 experiment)
benchmark/vocab/             fr.txt, en.txt: vocabulary files for the drug-name study (committed)
benchmark/tts/               sentences and 5/10-sentence paragraphs for the TTS measurement, listening texts (committed); benchmark/tts-samples/ (and session2/) = generated listening files (git-ignored)
src-tauri/models-manifest.json   model inventory (data, no code change to add a model)
scripts/                     PowerShell/Python helpers (env check, library/CLI builds, comparison, bootstrap CI, wer_by_language.py = WER per language and key terms per kind); TTS: tts_listening.py (--session 1|2), tts_overhead.py, tts_normaliser_check.py, tts_speaker_scan.py, convert_piper_voice.py; UI tests: ui_test_readalong.mjs, ui_test_packaged.mjs + cdp.mjs; packaging: stage-whisper-sidecar.ps1, offline_proof_rule.ps1 (+ .cmd wrappers, run by the owner)
vendor/, wav/, models, target/   git-ignored (downloads, builds, the owner's private audio; vendor/py-convert = isolated Python environment with `onnx` only, vendor/piper-mls-source = source files of the converted mls voice)
```

## 5. Environment facts and traps (Windows 11, Intel Core 7 150U, 23.6 GB, CPU only)

- **Dev server port is 1430**, not 1420: 1420 belongs to the owner's separate AssistantCabinetAI app. Never stop that process.
- **TLS interception by an antivirus** breaks the sherpa-onnx crate build download (`UnknownIssuer`). Run `scripts/fetch-sherpa-libs.ps1` once; it writes the git-ignored `src-tauri/.cargo/config.toml`. Model downloads work (native certificates). Never disable the antivirus or TLS checks.
- **whisper.cpp** is built from source by `scripts/build-whisper-cpp.ps1` into `vendor/` (CMake required; LLVM is not).
- **Support model**: `silero-vad` (MIT, 0.6 MB) is installed with `transcribe install silero-vad`; it is hidden from `list()` and from the engine pickers. `bench run --chunking vad` refuses to start without it.
- **Voices are licence-rated** (`docs/TTS_LICENSES.md`, D-039): a new voice needs its card read INCLUDING "Finetuned from", a rating, a reason and a stated phonemizer in the manifest, or the manifest test fails. Listening page: `python -I -X utf8 scripts/tts_listening.py`.
- **Models** live in `%APPDATA%\ai.assistantcabinet.speechlab\models` (all installed except whisper turbo). Clips in `...\recordings`.
- **Benchmarks need a calm machine.** The owner also runs Docker Desktop/WSL, which uses CPU. On this laptop the Windows firewall engine, DNS client and the security software alone keep the background load at 16 to 31 %, so `bench run` refuses to start above **30 %** average CPU or on battery (D-034, relaxed from 15 % by the owner's choice to mimic a client machine). Run `bench preflight` first and quote the recorded load with any speed figure. Never use `--force` for figures you intend to publish. Do not compile, test, record or take screenshots while a benchmark runs.
- **A running benchmark survives an interrupted session.** Before starting another one, check `tasklist` for `bench.exe`/`whisper-cli.exe` and the log file. Never run two benchmarks at the same time.
- **Use `--release`** for any speed or memory figure.
- **GitHub once returned HTTP 500** when creating a new branch from the M4 commit (I-031); creating the branch from `main` in the web UI worked, then a normal push.
- **Shell quoting**: very long inline shell/Python scripts containing apostrophes were rejected by the agent shell several times. Write script files and run them, or use small edits. In Python strings avoid backslash escapes such as `\b`.
- **Read generated test code** before running it, and validate any scoring rule on real engine output before trusting a number (five scoring defects were found this way, see PROJECT_LOG and D-030).
- **Read-along UI test**: `node scripts/ui_test_readalong.mjs [screenshot-dir]` against the app started with the remote-debugging switch (port 9222, page on 1430); it reloads the page, mutes the audio, never plays by itself, deletes only the WAV files it created. It needs the Piper ljspeech voice, and the Coqui and siwis voices for the last checks. Give the TTS buttons ids when adding another panel with a button of the same name (the speech-to-text panel also has a "Stop"). Stopping a `pnpm tauri dev` background task does not stop its children (`speechlab.exe`, `cargo`, `pnpm`, Vite `node`): find them by command line and stop only the ones from this repository.
- **Packaging (M7)**: before ANY `cargo build/test/run` or `pnpm tauri dev/build`, `src-tauri/binaries/` must hold the staged sidecar (`scripts/stage-whisper-sidecar.ps1` after `scripts/build-whisper-cpp.ps1`), else the build stops (I-064). `pnpm tauri build` takes 2 to 3 minutes once the release dependencies are compiled; the installer is `src-tauri\target\release\bundle\nsis\SpeechLab_0.1.0_x64-setup.exe`. Test an installer by a silent install into a scratch folder (`/S /D=<folder>`, `/D` last), run it with the DevTools switch, point the UI tests at the release origin with `SPEECHLAB_CDP_MATCH=tauri.localhost` (`scripts/ui_test_packaged.mjs`, `scripts/ui_test_readalong.mjs`), then remove the shortcuts, registry keys and folder it created. NEVER run the NSIS uninstaller without first moving the real `%APPDATA%\ai.assistantcabinet.speechlab` and `%LOCALAPPDATA%\ai.assistantcabinet.speechlab` folders away (the interactive uninstaller may offer to delete application data; the models are there). The tool shell named `bash` inside PowerShell is the WSL wrapper: use the Bash tool for shell scripts. The owner's `pnpm tauri dev` and the owner's other application share this machine: check `tasklist` and the listening ports before starting a dev server of your own, and stop only the processes you started (find them by creation time and command line).
- **Voice conversion**: Piper voices that sherpa-onnx does not ship are converted with `scripts/convert_piper_voice.py` (`check` first on an official package); packaging kind `local` means the voice cannot be downloaded (docs/TTS_LICENSES.md open point 5).
- UI testing without a human: run the app with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` and drive the page over the DevTools protocol (select the target whose url contains `localhost:1430`; the microphone permission prompt is a separate target; grant it with `Browser.grantPermissions`). Clean up any sample or clip created by tests.

## 6. How to run things

```bash
pnpm install
pnpm typecheck && pnpm test && pnpm build          # frontend checks
cd src-tauri && cargo test --lib                    # Rust unit tests (163; two use the real Piper siwis voice if installed)
pnpm tauri dev                                      # the app (port 1430)
powershell -ExecutionPolicy Bypass -File scripts/check-env.ps1     # machine and tool check

cd src-tauri
cargo run -q --example transcribe -- list           # models and install state
cargo run -q --example transcribe -- run <model-id> <fr|en> <file.wav> [repeat]

cargo build --release --example bench
./target/release/examples/bench.exe check           # dataset and audio checks
./target/release/examples/bench.exe preflight       # quiet machine? exit 0 yes, 2 no
./target/release/examples/bench.exe run --reps 1 --label <name>
./target/release/examples/bench.exe run --chunking vad --category fr-dictation,en-dictation --label <name>   # clips over 25 s cut at silences
./target/release/examples/bench.exe rescore --dir ../benchmark/results/<run>    # apply newer scoring rules
./target/release/examples/bench.exe summarize --dir ../benchmark/results/<run>

./target/release/examples/bench.exe run --vocab-dir ../benchmark/vocab --models whisper-cpp-small-q5_1 --label x          # initial prompt
./target/release/examples/bench.exe run --vocab-dir ../benchmark/vocab --hotwords-score 1.5 --models sherpa-parakeet-tdt-0.6b-v3-int8 --label x   # Parakeet hotwords (without --vocab-dir: beam-search control)
./target/release/examples/bench.exe postcorrect --dir ../benchmark/results/<run> --vocab-dir ../benchmark/vocab --preset strict --label x   # offline, no engine
./target/release/examples/bench.exe termstudy --dir ../benchmark/results/<baseline> [--against ../benchmark/results/<variant>] [--category a,b]   # key terms found, fixed vs broken words

python ../scripts/bootstrap_ci.py ../benchmark/results/<run> [--exclude en-it-05-owner]
python -I -X utf8 ../scripts/wer_by_language.py ../benchmark/results/<run>   # WER per language with intervals; key terms found per kind (aggregates only)
python ../scripts/compare_runs.py ../benchmark/results/<old> ../benchmark/results/<new>
python ../scripts/timing_study.py ../benchmark/results/<run with --reps 3>
python ../scripts/chunking_study.py runs|same|clips ...      # whole versus chunked, identity check, private clips (counts only)

cargo build --release --example tts
./target/release/examples/tts.exe voices                                  # installed voices (id = <package>:<speaker>)
./target/release/examples/tts.exe say <voice-id> <fr|en> "<text>"|@file.txt [--speed 1.0] [--repeat N] [--out-dir DIR] [--normalise on|off] [--whole 1] [--show-text 1] [--show-segments 1]   # every option takes a value
./target/release/examples/tts.exe measure --sentences ../benchmark/tts/sentences-fr.txt --lang fr --voices a,b --reps 3 --out-dir ../benchmark/results/<run> --label x [--compare 1]   # --compare: per-sentence and whole-text modes interleaved
python -I -X utf8 ../scripts/tts_overhead.py ../benchmark/results/<run>     # per-sentence overhead table from a --compare run
python -I -X utf8 ../scripts/tts_normaliser_check.py <stt model id> 3         # does the normaliser make Coqui speak numbers? (recogniser round trip)
python -I -X utf8 ../scripts/tts_listening.py --session 2                    # blind listening page of session 2
node ../scripts/ui_test_readalong.mjs                                         # read-along UI test (app started with the DevTools switch)
node ../scripts/ui_test_packaged.mjs --install-dir <folder> [--empty] [--no-mic]   # packaged-app test, 24 checks (SPEECHLAB_CDP_MATCH=tauri.localhost)

# package (from the repository root)
powershell -ExecutionPolicy Bypass -File scripts/stage-whisper-sidecar.ps1   # needed before any cargo build of the app package (I-064)
pnpm tauri build                                                              # NSIS installer, see README
python ../scripts/tts_summary.py ../benchmark/results/<run>    # table from the jsonl files
python -I -X utf8 ../scripts/tts_roundtrip.py ../benchmark/tts-samples <stt model id>   # machine intelligibility check (not naturalness)
```

## 7. Backlog (priority order; ready-made prompts in `docs/prompts/`)

| # | Task | Why | Prompt |
|---|---|---|---|
| 1 | ~~Clean re-run of the full benchmark~~ DONE 2026-10-07 (I-039 resolved) | | `docs/prompts/01-clean-benchmark-rerun.md` (kept for reference) |
| 2 | Owner listens to `en-it-05`, re-record if needed, re-run only that sample | all engines hear "543", script says "443" (I-033) | in next-tasks.md |
| 3 | ~~Timing study, 3 repetitions~~ DONE 2026-10-08 (I-012 partly closed; between-run offset unexplained) | | next-tasks.md T2 (kept for reference) |
| 4 | ~~Long audio, VAD chunking~~ DONE 2026-10-08 (D-035; open follow-ups: per-engine segment limit and output guard, I-041, I-035) | | `docs/prompts/02-long-audio-vad-chunking.md` (kept for reference) |
| 5 | ~~Drug-name handling~~ DONE 2026-10-08 (D-036, I-043, I-044, I-045; open: real SentencePiece vocabulary, free-text false-correction rate, owner's choice of technique) | | `docs/prompts/03-drug-name-handling.md` (kept for reference) |
| 7 | Detector false alarms for times ("10.30", "9h00"), sherpa-onnx thread usage | I-034, I-036 | small fixes |
| 8 | ~~M6 TTS laboratory~~ BUILT AND MEASURED 2026-10-08 (D-038 to D-041, I-048 to I-057); open: read-along highlight and French number normaliser (prompt 06b), measure and listen to `kristin`, a better free French voice (Piper `mls` conversion needs approval), legal reading of the `review` voices (postponed), French male voice | | `docs/prompts/05-tts-laboratory.md` (kept for reference); a prompt for the highlight and the normaliser is written once the owner chooses |
| 8b | ~~TTS read-along highlight, normaliser, `kristin`, French voice research~~ BUILT AND MEASURED 2026-10-08 (D-042, I-058 to I-062); session 2 done; open: number rewriting is now ON by default for every voice (D-043, coded; the read-along UI test was not re-run since); **before commercialisation: re-train siwis and gilles from the clean base checkpoint (GPU outside this machine, I-063)**; click-to-jump later if wanted | D-040, D-041, D-042 | `docs/prompts/06b-tts-readalong-and-normaliser.md` (kept for reference) |
| 9 | ~~M7 packaging validation~~ DONE for Windows 2026-10-08 (D-044, D-045; open follow-ups: I-065 one-line UI fix, I-067 credits view + installer licence page, I-070 static whisper.cpp rebuild, I-066, clean-machine test, signing; macOS needs a Mac) | plan section 9 | `docs/prompts/06-packaging-validation.md` (kept for reference) |
| 10 | ~~M8 final report and licensing table~~ DONE 2026-10-09 (D-047; `docs/SPEECH_ENGINE_EVALUATION.md`) | main deliverable | `docs/prompts/07-final-report.md` (kept for reference) |
| 11 | **ON HOLD (D-050) until the Knowledge Base is finished, then SP-R (re-baseline the plan).** Integration into AssistantCabinetAI: SP-0 (SpeechLab: freeze, decision sheet, practice-PC probe; may be run earlier if the owner asks) then SP-1 to SP-8 in AssistantCabinetAI tabs, as re-baselined | the owner's goal after M8; D-049 (proposed) | `docs/prompts/integration/` (README, `00-MASTER.md`, `SP-0` to `SP-8`); plan `docs/integration/03-integration-plan.md` |
| 12 | Decide the GPL phonemizer strategy and the licence of the code; credits view and installer licence page (I-072, I-067); legal reading list in the report, section 9.6 | blocks any distribution that synthesises speech | decisions Q-01, Q-09 of the plan |
| 13 | Re-train the French voices siwis and gilles from the clean base checkpoint (GPU outside this machine), blind listening again (D-043, I-063) | commercial-release blocker for French TTS | none yet |
| 14 | ON HOLD: macOS validation on a Mac (`docs/MACOS_VALIDATION.md`) (I-075); clean Windows machine and Windows 10 test; code signing | compatibility claims are NOT VERIFIED | none yet |
| 15 | Widen the STT evidence: more speakers, noise, the practice PC, whisper.cpp large-v3-turbo and accelerated builds, streaming (I-074) | the recommendation is provisional until then; the practice-PC part is SP-0 | SP-0 for the PC; others none yet |

## 8. Session protocol

Start: `AGENTS.md` → this file → last entries of `docs/PROJECT_LOG.md` → open items in `docs/ISSUES.md` → `git status` and `git log` (read-only) → check no benchmark is running (`tasklist`).

During: small steps, tests with every change, document failures as you go.

End:
1. Append a dated entry to `docs/PROJECT_LOG.md` using the headings **Done / Verified / Failed or surprises / Not verified / Next**.
2. Update `docs/DECISIONS.md`, `docs/ISSUES.md` and, if the state changed, sections 3 and 7 of this file and `README.md`.
3. Check the identity rule: search the repository for AI or vendor names (none must appear) and make sure no private content was written.
4. Give the owner the exact git commands (branch, `git add` of explicit paths, one-sentence English commit, push) and **do not run them**. Check `git status` first so the list is complete (earlier hand-offs forgot `README.md`).
5. Tell the owner, in French: what was done, what is verified, what is not, limits, and the proposed next step.

## 9. Vocabulary

- **WER / CER**: word / character error rate. Micro WER = total errors over total reference words.
- **RTF**: real-time factor = processing time / audio duration (below 1 is faster than real time).
- **Cold start**: the run that includes loading the model. whisper.cpp runs as a new process each time, so it is always cold.
- **Decoding**: greedy (1 candidate) or beam search (several). Always report which one was used (D-018).
- **Critical flag**: a changed, lost or added number, unit, negation or weekday/month, or a missing key term (drug names are critical). Heuristic, about 15 % false alarms on times; keep expected/found text for human review.
- **Scoring version**: scoring rules are versioned; old results can be re-scored from stored transcripts (D-030).
- **Suspect sample**: most engines agree with each other but not with the reference: the reading or the reference may be wrong (I-033).
