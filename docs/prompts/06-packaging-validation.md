# Prompt 06 — M7: packaging validation (Windows build, macOS steps)

STATUS (2026-10-08): DONE for Windows (project log entry "M7", D-044, D-045, I-064 to I-070); macOS only as a
checklist (`docs/MACOS_VALIDATION.md`, NOT VERIFIED). Open from this task: the cancelled-transcription banner
(I-065), a credits view and installer licence page (I-067), a static whisper.cpp rebuild (I-070), the web view's
background connection (I-066), the interactive uninstaller and a clean-machine test. Kept for reference.

Paste everything below the line into a new chat session. Prerequisites: the M6 and M6d work (D-038 to
D-042) is committed and no benchmark is running. This task needs no clip and no listening. It builds an installer
on this machine (long compile, large output in `target/`, git-ignored): ask the owner before running
`pnpm tauri build` for the first time and say how long and how big it is. No Mac is available: macOS
can only be prepared on paper and stays NOT VERIFIED.

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
3. The last entries of `docs/PROJECT_LOG.md` (M6, M6c, M6d), `docs/DECISIONS.md` D-004, D-012, D-017,
   D-034, D-038, D-039, D-041, D-042, `docs/TTS_LICENSES.md`, `docs/TTS_LISTENING_NOTES.md`, `docs/ISSUES.md` (I-009, I-010, I-024, I-026,
   I-051, I-059, I-060), `Plan.md` (packaging and compatibility sections) and `docs/prompts/next-tasks.md` section T7.

## Step 0 — what changed since the notes were written (read, no action)
The owner's listening session 2 is DONE and recorded (`docs/TTS_LISTENING_NOTES.md`). The French voices the owner keeps
(Piper siwis `review`, gilles `excluded`) have restricted parent checkpoints: by the owner's decision (D-043, amending D-041)
they stay available for development, evaluation and non-commercial use and must be re-trained from the clean base before
any commercial release (I-063). Models are never bundled, the app must keep showing their rating, and the packaging report
must list this as a blocker for a commercial build. Do not start the re-training in this session unless the owner asks.
The rewriting of numbers is ON by default for every voice (owner's approval, D-043, coded in M6e addendum 3). The read-along
UI test was NOT re-run after that change: re-run `node scripts/ui_test_readalong.mjs` on the packaged app (expect 33 of 33,
the phonemizer-voice check now expects the box checked).

## Context
New since the M6 work (D-042): the TTS tab shows a read-along view (sentence highlight, auto-scroll), a text
normaliser rewrites numbers for character-based voices, and one voice (Piper `mls`) is converted locally with
`scripts/convert_piper_voice.py` (packaging kind `local`: it cannot be installed from the app, so a packaged build
must not offer it as a default; document how a product would obtain such a voice). The UI test
`node scripts/ui_test_readalong.mjs` (33 checks) can be pointed at the packaged app through the remote-debugging switch.

Everything so far was run from `pnpm tauri dev` and from `cargo` examples. Nothing has been packaged:
`src-tauri/tauri.conf.json` has `"bundle": { "active": false }`. Facts to check, not to assume:
- sherpa-onnx is linked statically (`sherpa-onnx-v1.13.8-win-x64-static-MT-Release-lib`), so no extra
  DLL is expected, but the TTS voices need the `espeak-ng-data` folder that sits INSIDE each model
  package in the models folder (so it is downloaded, not bundled).
- whisper.cpp runs as the external program `whisper-cli.exe` built into `vendor/`; the code looks for a
  "future sidecar location" next to the executable first (see `whisper_cpp.rs`). The Tauri sidecar
  mechanism (`externalBin`) has not been configured.
- Models live in `%APPDATA%\ai.assistantcabinet.speechlab\models` (override: `SPEECHLAB_MODELS_DIR`);
  generated speech in `...\tts-output`; clips in `...\recordings`. Models must NOT be bundled.
- The Content Security Policy is `null` (development setting) and the microphone permission was only
  seen in the development WebView origin (I-024).
- The espeak-ng phonemizer (GPL-3.0) is linked in whenever TTS is present (D-012, I-051, I-054): the
  installer of a build that contains it carries that licence obligation. Do not decide it; document it.
- Voices are rated for commercial use (D-039, `docs/TTS_LICENSES.md`). No voice rated `review` or
  `excluded` may be a default of a COMMERCIAL build (the owner suspended this rule for siwis and gilles until commercialisation, D-043), models are downloaded not bundled, and the credits
  required by `attribution` voices (CC BY 4.0, BSD-3-Clause) need a place in the app (an About/credits
  view) and in the installer. Check what the packaged app shows.
- An antivirus intercepts TLS on this machine (I-009); it may also flag a new unsigned executable.

## Task
1. **Read-only checks**: `git status`, `git log --oneline -5`, `tasklist` (no `bench.exe`,
   `whisper-cli.exe`, `tts.exe` running), free disk space (the release build of a Tauri app with these
   libraries can take several GB), `pnpm tauri --version`, whether WiX or NSIS tooling is installed
   (do not install anything: propose commands to the owner).
2. **Propose the packaging design before building**, in French, and wait for the owner's go if a
   download or install is needed: which bundle format (NSIS `.exe` or MSI), where `whisper-cli.exe` goes
   (Tauri `externalBin` sidecar, with the target-triple suffix), how the app finds the models folder, what
   happens on first start with no model (the app must say so and offer the install buttons, not crash).
3. **Build** with `pnpm tauri build` (owner's approval first). Measure and record: build time, installer
   size, installed size, whether `whisper-cli.exe` ends up next to the executable, cold start of the app
   (time to a visible window), memory at idle. Everything else as measurements, not impressions.
4. **Run the installed/packaged app** and check, one by one, with evidence: window opens; model list
   shows; an installed STT model transcribes a short clip; whisper.cpp engine works through the sidecar;
   the microphone permission prompt appears in the RELEASE origin and the choice persists after a restart
   (closes or documents I-024); a TTS voice generates a WAV (this checks that the `espeak-ng-data`
   folder is found from the packaged app, that the read-along view shades the sentences while a muted playback runs, and that the Coqui voice speaks numbers: `scripts/ui_test_readalong.mjs` covers most of it); Cancel works. Drive the UI through the DevTools protocol as in
   `docs/HANDOFF.md` section 5 (the remote-debugging switch also works for a release build) and never touch
   port 1420 or the owner's other processes. Clean up anything the tests create.
5. **Offline proof** (the plan requires it): with the owner's agreement to switch the network off (or
   a firewall rule they create), repeat one transcription and one synthesis and record that they work
   without network. Do not disable the antivirus or TLS checks.
6. **Antivirus and signing**: record whether the unsigned installer or executable is flagged or blocked
   here; do not try to evade it. State what code signing would need (certificate, cost, SmartScreen
   behaviour) as NOT VERIFIED.
7. **macOS (Apple Silicon and Intel)**: no Mac is available. Write the exact validation steps as a
   checklist file (`docs/MACOS_VALIDATION.md`): prerequisites, `pnpm tauri build` for both targets, the
   sherpa-onnx macOS libraries and `scripts/fetch-sherpa-libs.ps1` equivalent, building `whisper-cli`
   with Metal or Core ML options, `externalBin` suffixes, WKWebView differences (audio decoding of
   Ogg/Opus and MP3, microphone permission text in `Info.plist`, I-024/I-026), notarisation and
   hardened runtime, where models are stored, espeak-ng data location. Mark every line NOT VERIFIED.
8. **Tighten what the packaging reveals**, with the owner's approval for each change: the CSP (replace
   `null` by a real policy and re-test the UI), the capabilities file, and the first-start experience.
   Any code change needs tests and a log line.
9. **Document** (all English): `docs/PROJECT_LOG.md` (Done / Verified / Failed or surprises / Not
   verified / Next), `docs/DECISIONS.md` (bundle format, sidecar, CSP, what is and is not bundled),
   `docs/ISSUES.md`, `README.md` (how to build and install the package), sections 3 and 7 of
   `docs/HANDOFF.md`, `docs/prompts/README.md`, `docs/MACOS_VALIDATION.md`.
10. **Identity and privacy check**: search the repository for AI or vendor names (none allowed; the
    process list recorded by `bench preflight` can contain an editor name, replace it by a neutral label
    in any result folder's `config.json`), no private content, and `git status` must not list the
    installer, `target/` output or any model file.
11. **Final message to the owner, in French**: what was done, verified, not verified, limits, the exact
    git commands for the owner to run (never run them yourself; check `git status` first so no path is
    forgotten; one English sentence per commit message, imperative, about 72 characters, no trailer), and
    prepare the prompt file for the following step: M8, the final report and licensing table
    (`docs/prompts/07-final-report.md`, from `next-tasks.md` T8).

## Reminders
- Honest reporting: say what is verified and what is not. No invented numbers.
- Do not commit. Do not install software or download anything without asking. Do not touch the
  owner's other app (port 1420) or their Docker/WSL work.
- Very long inline shell or Python scripts with apostrophes were rejected by the agent shell: write
  script files and run them. `python -I` ignores `PYTHONIOENCODING`: use `python -I -X utf8`.
- Never run two benchmarks at once; do not compile, test or record while one runs. A packaged-app
  build is a heavy compile: never during a measurement.
