# Issues

Record every bug, blocker, or surprising behavior. Keep resolved items.

| ID | Status | Area | Summary |
|---|---|---|---|
| I-001 | Resolved | scripts | `check-env.ps1` printed `MSVC` instead of the toolset version |
| I-002 | Resolved | environment | CMake installed (4.4.4) for M3; LLVM not needed with the external-process design (D-017) |
| I-003 | Open | licensing | espeak-ng GPL-3.0 in sherpa-onnx Piper/Kokoro TTS path (see D-004) |
| I-004 | Open | models | License unknown for French streaming zipformer / Kroko models (HF returned 401) and Piper `miro`, `tjiho` voices (model card 404) |
| I-005 | Open | environment | `python` is the Microsoft Store build; verify behavior when scripts are run from services/CI |
| I-006 | Resolved | dev environment | Port 1420 conflict with the AssistantCabinetAI desktop dev server |
| I-007 | Open | tooling | pnpm 12 added `minimumReleaseAgeExclude: vite@8.3.3` automatically |
| I-008 | Resolved | verification | `transcribe` command exercised through the real window in M2 |
| I-009 | Mitigated | environment | Antivirus TLS interception breaks the sherpa-onnx-sys build download (`UnknownIssuer`) |
| I-010 | Open | licensing | espeak-ng (GPL-3.0) is linked into the STT-only binary with the default sherpa-onnx libs; no-tts build avoids it |
| I-011 | Open | engine | sherpa-onnx offline decode cannot be cancelled mid-run |
| I-012 | Open | benchmarking | Large run-to-run timing variance (same input 1306 ms vs 2651 ms) |
| I-013 | Open | engine | sherpa-onnx prints resampler/debug lines to the console |
| I-014 | Open | data | No verified-license French test audio yet; `fr.wav` license unstated (test use only) |
| I-015 | Mitigated | audio input | Non-WAV files with a `.wav` extension (MP4/M4A, Ogg) now get a clear error; decoding them is still unsupported (planned M4) |
| I-017 | Open (chunking tested, see update) | engine | Canary logs "first generated token is <|endoftext|> ... (issue #3919)" on a 45.8 s recording; output was still produced |
| I-016 | Explained (partly) | evaluation | Owner-reported: Whisper tiny transcribed the owner's recording very badly. Cause for sherpa-onnx's Whisper tiny: repetition loop on a 45.8 s file (see I-018); not a verdict on whisper.cpp |
| I-018 | Open (chunking recovers the ending, not the loops) | engine | sherpa-onnx Whisper tiny (greedy) falls into a repetition loop on a recording longer than 30 s |
| I-019 | Confirmed, not fixed by chunking | evaluation | Canary int8 dropped the final spoken sentence of the 45.8 s owner recording (owner listened: it is spoken) |
| I-020 | Open | design | whisper.cpp as external process = cold start on every run |
| I-021 | Open | packaging | `whisper-cli` is located in `vendor/` in dev only; must become a bundled sidecar (M7) |
| I-022 | Mitigated | environment | `vswhere.exe` missing: Visual Studio CMake generator unusable; NMake used instead |
| I-028 | Mitigated | audio input | Owner's microphone input seems to saturate (recording quality mediocre); a level/clipping indicator was added, cause not confirmed |
| I-029 | Partly measured (M5e) | engines | Long audio (45 s private clip, 1969 speech MP3): Canary truncates, Whisper tiny (both runtimes) repeats near the end; chunking/VAD needed (see I-018, I-019) |
| I-031 | Mitigated | git hosting | GitHub returned HTTP 500 when creating any new branch pointing at the M4 commit; creating the branch from main in the web UI worked |
| I-032 | Open | ui | The dataset recorder shows the benchmark path with a ".." segment (cosmetic) |
| I-039 | Resolved | benchmark | First full run was disturbed by a heavy task; clean re-run done (855/855 transcripts identical, new speed figures at about 20 % background CPU) |
| I-033 | Open | dataset | Suspect sample en-it-05: all engines hear 543, the script says 443 (listen, maybe re-record) |
| I-034 | Open | scoring | Detector false alarms on times written "10.30", "3.30", "9h00" (about 15 % of reviewed flags) |
| I-035 | Open (reproduced with chunking on one private clip) | engines | Canary int8 emitted runaway garbage on one slow recording; product needs an output guard |
| I-036 | Open | performance | sherpa-onnx uses 7 to 9 busy cores with 4 threads configured |
| I-037 | Open | performance | whisper.cpp small is slower than real time on this CPU (RTF 1.6 to 1.9) |
| I-038 | Info | benchmark | One speaker, 95 sentences, 1 repetition, CPU only; sampling interval via bootstrap |
| I-041 | Open | engines | Canary int8 degrades on pieces of about 24 s: a whole sentence dropped on `en-dict-01` chunked at 25 s, half the words and 63 s of runtime on a 45.8 s private clip |
| I-043 | Open | engines | Parakeet hotwords need a surrogate SentencePiece vocabulary (the download has none); score 1.5 loses the elision ("d amoxicilline"), score 3.0 inserts vocabulary words into unrelated sentences |
| I-044 | Open | engines | whisper.cpp initial prompt: 25 to 40 % slower, slightly worse on sentences without key terms, harmful for tiny, no reliable gain on drug names |
| I-045 | Open | scoring / safety | Dictionary post-correction can silently replace a correct rare word; false-correction rate on free text unknown (measured only on 95 sentences) |
| I-046 | Open | dataset import | `bench import` resamples non-16 kHz clips by linear interpolation, never compared with a better resampler; only PCM WAV input is accepted (m4a/mp3/Ogg must be converted first) |
| I-047 | Open | privacy | Private runs: `summary.md` prints the reference and best output of the hardest samples and `runs.jsonl` holds every text; the folder is git-ignored but nothing from it may be quoted in a document |
| I-048 | Open | TTS performance | Kokoro int8 is slower than real time on this CPU (RTF 1.55 to 2.06); Piper is 13 to 29 times faster than real time |
| I-049 | Open | TTS voices | Voice metadata is thin: Piper speaker gender undocumented, 904-speaker package listed 12 at a time, no commercially usable French male voice identified yet |
| I-050 | Open | TTS engine | Kokoro French logs unknown-phoneme (hyphen) messages; cause and audible effect not isolated |
| I-051 | Open | licensing | Any build that synthesises speech links the full sherpa-onnx libraries, which contain the GPL-3.0 phonemizer (extends I-003, I-010, D-012) |
| I-052 | Open | TTS licensing | Fine-tuned Piper voices inherit an unclear licence: siwis (from Lessac, research licence) = review; gilles (from Ryan, CC BY-NC-SA) = excluded |
| I-053 | Open | TTS licensing | Kokoro training audio includes output of closed TTS services (terms unread): rating review |
| I-054 | Open | TTS licensing | The GPL phonemizer code stays inside the sherpa-onnx libraries even when a character-based voice never calls it |
| I-055 | Mitigated (M6d) | TTS quality | Character-based voices (Coqui css10) do not speak digits: the text normaliser is built and on by default for that voice (28/36 and 30/36 key numbers heard instead of 0/36); close after the owner's listening |
| I-056 | Open | TTS measurement | Synthesis is not deterministic: regenerated audio differs, so recogniser round trips need repetitions |
| I-057 | Open | TTS quality | Audible hiss on all three Kokoro voices (owner's blind listening); cause not isolated, Kokoro deprioritised |
| I-058 | Open | TTS quality | The Coqui voice stays weak even when it speaks the numbers (date sentence still mangled by the voice itself) |
| I-059 | Open | UI | The application has no dark theme; only the read-along block follows the system setting |
| I-060 | Open | TTS voices | Piper `mls` (converted locally) is weak by a recogniser proxy (74 of 125 speakers at 90 % word error rate or worse) and cannot be installed from the app |
| I-061 | Open | TTS normaliser | Known limits: acronyms, gender of "un", Roman numerals, ambiguous "10.30" and "5 100", English untested by ear |
| I-063 | Open, accepted for now (owner, D-043) | TTS licensing and quality | French: the voices judged good (siwis, gilles) have restricted parent checkpoints, every clean French voice is judged unsatisfactory; the owner keeps siwis and gilles for now and will re-train them from the clean base before any commercial release (BLOCKER for commercialisation) |
| I-062 | Open | TTS read-along | Sentence segmentation is a heuristic (line breaks inside a wrapped paragraph split it, short abbreviation list) |
| I-064 | Open (workaround) | packaging | `cargo build/test/run` of the app package fails without the staged sidecar files (`scripts/stage-whisper-sidecar.ps1`) |
| I-065 | Open | UI | A cancelled transcription shows "operation cancelled" in the red error banner (the TTS panel ignores it); fix proposed, not approved |
| I-066 | Open | privacy | The WebView2 runtime keeps one HTTPS connection to a Microsoft address while the app runs, even with the app's own processes blocked; purpose NOT VERIFIED |
| I-067 | Open | licensing / UI | No credits view in the app and no licence page in the installer (CC BY 4.0, BSD-3-Clause, GPL-3.0 notices) |
| I-068 | Open | packaging | Unsigned installer and executables: nothing blocked here; SmartScreen and code-signing requirements NOT VERIFIED |
| I-069 | Open | packaging | Installer side effects: Desktop and Start Menu shortcuts, a registry key left after a silent uninstall; interactive uninstaller NOT tested |
| I-070 | Open | packaging | The whisper.cpp sidecar imports the Visual C++ runtime (MSVCP140, VCRUNTIME140, VCOMP140), which the installer does not carry; clean-machine behaviour NOT VERIFIED; static rebuild proposed |
| I-071 | Resolved (M8) | documentation | `README.md` had a bell and a line break where two backslash sequences of a Windows path had been turned into control codes; repaired |
| I-072 | Open | licensing | The SpeechLab repository has no `LICENSE` file (the application code has no stated licence); the owner decides |
| I-073 | Open | licensing | Licence readings still missing: training-data provenance of the NVIDIA models, redistribution terms of the Visual C++ runtime / WebView2 bootstrapper / installer builder, the five MPL-2.0 crates, the Whisper weights' MIT versus Apache-2.0 discrepancy |
| I-074 | Open | evidence gaps | Never run: whisper.cpp large-v3-turbo, accelerated builds, a second or weaker machine, noisy or conversational speech, other speakers, streaming recognition; the M8 recommendation stays provisional until they are measured |
| I-042 | Open | benchmarking | Median inference time of an unchanged code path differed by 1 to 48 % between two runs (clean full run versus `short-vad`), larger than the timing study suggested |
| I-030 | Open | scoring | Swiss number words (septante, huitante, nonante) are not folded to digits |
| I-024 | Closed for Windows (M7), macOS open | audio input | Microphone: WebView2 permission prompt on first use; release origin and persistence after a restart VERIFIED on Windows (M7); macOS behaviour NOT VERIFIED |
| I-025 | Open | audio input | Recordings are about 1.5 % (~0.1 s on 6 s) shorter than the time held; cause not isolated |
| I-026 | Open | audio input | Non-WAV import relies on the platform WebView decoders (Ogg/Opus on macOS unverified) |
| I-027 | Resolved | privacy | Saved clips became invisible and undeletable after a UI reload (fixed: clips are listed from disk) |
| I-023 | Open | evaluation | sherpa-onnx Whisper tiny says "Demandez vos puto" while whisper.cpp tiny (same weights, even greedy) is correct on `fr.wav`; cause unknown |

---

### I-001 — MSVC version not printed
- Symptom: `OK MSVC MSVC`.
- Cause: glob stopped one level too high (`...\VC\Tools\MSVC` instead of `...\MSVC\*`).
- Fix: glob changed to `...\VC\Tools\MSVC\*`. Re-run on 2026-10-06: now prints `MSVC 14.44.35207`. VERIFIED.

### I-002 — CMake / LLVM missing
- Impact: blocks whisper.cpp builds (M3) only. `whisper-rs-sys` needs CMake **and** libclang (bindgen); a sidecar build of whisper.cpp needs only CMake.
- Action: ask the owner before installing (`winget install Kitware.CMake`, `winget install LLVM.LLVM`).

### I-003 — espeak-ng GPL-3.0
- See D-004. Needs legal review and alternatives analysis in M6/M8.

### I-004 — Unknown licenses
- Re-check with a browser or alternate source in M8; treat as NOT usable until confirmed.

### I-005 — Store Python
- Low priority; note for benchmark scripts.

### I-006 — Port 1420 already in use
- Symptom: `Error: Port 1420 is already in use`, Tauri reports `beforeDevCommand` failed.
- Cause: the AssistantCabinetAI desktop app runs its own Vite dev server on 1420.
- Fix: SpeechLab uses 1430 (D-008). Do not kill the other process.

### I-007 — pnpm release-age protection
- pnpm 12 wrote the exclusion for Vite 8.3.3 into `pnpm-workspace.yaml` on its own (it appears to be its protection against very recently published versions; exact behaviour not investigated). Review when upgrading dependencies; the file is committed on purpose so installs are reproducible.

### I-008 — Unclicked transcribe path
- Resolved in M2: transcription, install and error paths were driven in the real Tauri window (WebView2 DevTools protocol, local only). Results are in PROJECT_LOG.

### I-009 — TLS interception breaks the crate build download
- Symptom: `Failed to download sherpa-onnx archive ... invalid peer certificate: UnknownIssuer` during `cargo build`.
- Cause: Avast Web/Mail Shield re-signs HTTPS traffic with its own root CA (present in Windows, absent from rustls bundled roots).
- Workaround: `scripts/fetch-sherpa-libs.ps1` (curl.exe + official SHA-256) and git-ignored `src-tauri/.cargo/config.toml` setting `SHERPA_ONNX_ARCHIVE_DIR`. Runtime model downloads use native certs and work.
- Not done on purpose: disabling the antivirus or TLS verification.

### I-010 — espeak-ng in the STT binary
- See D-012. Needs legal review; mitigation path = no-tts libs for STT-only builds.
- M7 note: the packaged `speechlab.exe` (31.4 MB) links the full static sherpa-onnx libraries, so the NSIS installer of this build contains the GPL-3.0 phonemizer code; nothing in the installer states it yet (I-067). Not decided here (D-044).

### I-011 — No mid-run cancellation for sherpa-onnx
- Options for later: run inference in a child process and kill it, or accept "cancel = ignore result". Decide in M4 when recordings get long.

### I-012 — Timing variance
- Cause not investigated (laptop CPU boost/thermal, background load, antivirus). M5 must run N repetitions, record median and spread, fix the power mode, and log system state.

### I-013 — Console noise from sherpa-onnx
- Cosmetic. Revisit if the crate exposes a log hook.

### I-015 — Non-WAV input
- Symptom: `audio error: cannot open ...Coucou.wav: Ill-formed WAVE file: no RIFF tag found`.
- Cause (verified): the file is an MP4/M4A container (`ftypisom`) renamed to `.wav`; the `.ogg` voice note is Ogg.
- Workaround: convert to PCM WAV with Audacity or VLC (free). Mono 16 kHz is ideal but any channel count and sample rate is handled.
- Done (owner chose option a): `wav.rs` detects MP4/M4A, Ogg, FLAC, MP3, WebM and RF64 from magic bytes and says so ("... is MP4/M4A (AAC), not a PCM WAV file ... Convert it to WAV first"). Verified on the owner's real `.ogg` file and by unit tests. Full decoding remains pending for M4.
- Options (original analysis): (a) detect the real format from magic bytes and report "this file is M4A/Ogg, not WAV" (small change); (b) decode M4A/AAC, Ogg/Opus, MP3 with an open-source Rust decoder (e.g. symphonia, MPL-2.0; license to check). Decision pending, proposed for M4.

### I-016 — Whisper tiny vs Canary on owner recordings
- M3 update: the bad result came from sherpa-onnx's Whisper tiny, which looped on the 45.8 s file (I-018). whisper.cpp tiny on the same file produced a full-length transcript (word-level disagreement with Canary 0.65, which is not a WER against truth).
- Informal owner observation only. To be replaced by measured WER/CER in M5 with several Whisper sizes (tiny, base, small, large-v3-turbo) on both engines. Do not cite as a conclusion.

### I-017 — Canary end-of-text warning on longer audio
- Observed after the owner converted `Coucou.wav` to a real WAV (45.8 s, private content, not reproduced in the docs): Canary int8 transcribed it (inference 20.9 s, RTF 0.455, 4 threads, debug build CLI) but sherpa-onnx logged that the first generated token was `<|endoftext|>` and a fallback was used (upstream issue #3919). Output quality on this file was not measured. Check in M5 whether long audio needs chunking/VAD (Canary-flash is documented upstream for short segments; not verified here).

### I-014 — Test audio licensing
- Plan: generate synthetic speech in M6 (TTS) and/or source public-domain or CC audio (e.g. LibriVox for fr/en) with provenance recorded. A Swiss French source is still unidentified.

### I-018 — Repetition loop in sherpa-onnx Whisper (greedy)
- Observed on the owner's 45.8 s recording (single run, content private): the transcript ends with the same phrase repeated many times and has only 71 words versus ~100 for the other engines. Whisper models work on 30 s windows; sherpa-onnx's offline Whisper handling of longer audio and its greedy search have no repetition or fallback control (cause not confirmed).
- Mitigation ideas for M5: cut long audio into <30 s chunks with VAD (sherpa-onnx ships Silero VAD) before Whisper; compare against whisper.cpp which uses temperature fallback.

### I-019 — Dropped ending in Canary output
- CONFIRMED by the owner (2026-10-07): the recording does end with a closing sentence (a goodbye followed by the addressee's first name and "ciao"). Canary int8 omitted it; whisper.cpp tiny kept it but misheard the first name. The name is deliberately not written here (personal data).
- Consequence: Canary's "fairly good" impression hides a truncation on a 45 s file. Do not treat the earlier informal result as a pass. Test chunking/VAD (M5) and shorter audio before concluding.
- Canary's transcript stops at "... je pense." while whisper.cpp's continues with a closing sentence. Unknown which is right. Owner: listen to the last ~10 seconds of the recording and say whether that sentence is really spoken. If yes, Canary needs chunking for audio beyond a few tens of seconds (links to I-017).

### I-020 — Cold start on every whisper.cpp run
- Measured load times 82 to 243 ms for tiny (fp16). Larger models will cost more. Report load and inference separately (done). If it matters, consider a persistent worker.

### I-021 — Locating `whisper-cli`
- Order: SPEECHLAB_WHISPER_CLI, next to the app executable, then `vendor/whisper.cpp/build/bin`. The last one only exists on this dev machine. M7 must bundle the binary (Tauri sidecar) per OS/arch.

### I-022 — No vswhere.exe
- `vswhere.exe` is not present although MSVC 14.44 and the Windows SDK are. `vcvars64.bat` printed a vswhere error but still set up `cl`. If a Visual Studio generator is ever needed, repair the Build Tools installation (Visual Studio Installer).

### I-023 — Same weights, different output across runtimes
- On `fr.wav`: whisper.cpp tiny (fp16 ggml) correct; sherpa-onnx Whisper tiny (ONNX) wrong in 3/3 runs, even though both decode greedily when whisper.cpp is set to beam 1. To investigate in M5 with more samples and, if useful, the int8 vs fp32 ONNX files and whisper.cpp quantised models.

### I-028 — Microphone saturation
- Owner report: quality "mediocre", probably the microphone saturating. The app now shows peak level and clipped-sample share after each recording. If clipping is confirmed: lower the Windows input volume (Settings, System, Sound, Input), keep a hand's width from the microphone, or try the USB audio device. AGC is intentionally off in the app, so the Windows level is what matters.

### I-029 — Long audio behaviour
- Observed informally on two clips longer than 30 s. Plan for M5: split audio into segments of under ~25 s on silences (sherpa-onnx ships Silero VAD) before every engine, and compare "whole clip" versus "chunked" runs. The 1969 MP3 has unknown license: use only locally, never commit it; look for a clearly public-domain version (NASA) if it is wanted in the dataset.

### I-024 — Microphone permission
- Observed: the first `getUserMedia` shows a native WebView2 prompt (Block / Allow). In my tests the permission was pre-granted through the debugging protocol. To check manually: restart the app and see whether it asks again; test the `pnpm tauri build` output (origin `http://tauri.localhost`); if capture is silent or blocked, check Windows Settings, Privacy, Microphone ("let desktop apps access your microphone").
- **Update M7 (2026-10-08), Windows, installed release build, VERIFIED**: in the release origin `http://tauri.localhost` the permission state starts at `prompt`; the first `getUserMedia` opens the native dialog ("http://tauri.localhost souhaite / Utiliser vos microphones / Bloquer / Autoriser", a separate `edge://permission-request-dialog/` page); after "Autoriser" the capture works (3 s clip, 16 kHz mono 16-bit, stored; with the strict CSP too) and after quitting and restarting the app the state is still `granted` (no new dialog). The "Allow" click was made through the debugging protocol on the dialog's own button, so a real hand on a real mouse was not involved: the owner can repeat it once (reset: the choice is stored in the web view profile `%LOCALAPPDATA%\ai.assistantcabinet.speechlab\EBWebView`). The decision is per origin: the development origin `localhost:1430` has its own. Status: closed for Windows; macOS (WKWebView, `NSMicrophoneUsageDescription`) stays NOT VERIFIED, see `docs/MACOS_VALIDATION.md` section 5.

### I-025 — Recording shorter than wall time
- 6.0 s held produced 5.92 s (after the flush fix; 5.72 s before). Likely start/stop edge effects. Acceptable for dictation; re-check in M5 if timing alignment matters.

### I-026 — Import depends on WebView codecs
- Verified only with Ogg/Opus on Windows. Options for macOS and others: bundle ffmpeg as a sidecar (LGPL/GPL build choices to review) or a Rust decoder (symphonia has no Opus).

### I-027 — Orphan clips after reload (resolved)
- Cause: the clip list was React state only. Fix: `list_clips` at startup, lazy playback via `read_clip`, deletion via `delete_clip`, all confined to the store directory (unit-tested).

### I-030 — Swiss number words
- A Swiss French speaker says "septante", "huitante", "nonante". Whisper/Canary usually output digits, but when they output words the scoring does not know them. Cheap to add to `numbers.rs` (70, 80 in some cantons, 90) once a Swiss clip shows it matters. Not verified on any real output yet.

### I-031 — GitHub 500 on creating branches (2026-10-07)
- Symptom: `git push -u origin milestone/m5-benchmark` failed 3 times with `remote: Internal Server Error` (different Request IDs) after the pack was uploaded. GitHub status showed no incident; `git ls-remote` worked, so network and authentication were fine; the local repository passed `git fsck`.
- Bisect by the owner: pushing the already-known commit 70323f5 (zero objects to send) to a new branch name also failed with 500; a different branch name (`m5-benchmark`) and HTTP/1.1 changed nothing. Creating a branch from `milestone/m4-audio-compare` in the web UI was refused too, but creating it from `main` worked.
- Conclusion (partly inferred): the failure is tied to creating NEW refs that point at the M4 commit or its descendants, not to the content size, the name or the protocol; the existing `milestone/m4-audio-compare` ref (created earlier) is fine. Root cause on GitHub's side is unknown.
- Workaround: create the branch from `main` in the web UI (it then points at the merge of M3), then `git push -u` the local branch as a fast-forward update. Request IDs kept for a support ticket if it recurs: FE18:1A139B:2560149:240A6CB:6AC6617C, F73B:246D9C:25B96BE:245E261:6AC6619D, FFBA:2C0CBC:25B0D98:2458FDE:6AC661B7.
- Safety net used: `git bundle create ../SpeechLab-m5.bundle milestone/m5-benchmark` (local backup of the whole branch).

### I-032 — Unnormalised benchmark path in the UI
- `dataset::default_root()` joins `CARGO_MANIFEST_DIR` with `..`, so the path is displayed with `..`. Harmless; canonicalise when the runner is written.


### I-033 — Suspect sample: `en-it-05-owner` ("Open port 443 on the firewall.")
- All 9 configurations transcribe "port 543" (and one produced garbage). When every engine agrees against the reference, the speaker most likely said something else (for example "five forty-three"). Not an engine error until verified.
- Action (owner): listen to the recording in the dataset recorder; if it does not say "four forty-three", record it again (the recorder replaces the sample) and re-run only that sample. Until then it is reported in the "hard or suspect samples" section and excluded from the second bootstrap table.

### I-034 — Known false alarms of the critical-error detector
- Time written with a dot or with ":00": "10.30" and "3.30" against "10 30"/"3 30", and "9h00" against "9 heures". Of the 23 critical-severity flags reviewed by hand (Parakeet, Canary, Whisper small beam 5), 3 were false alarms of this kind, 1 concerns the suspect sample, the rest were real errors (drug names, a lost digit). So precision is roughly 80 to 85 % on this small review and recall is unknown. Do not present flag counts as exact.
- Possible fix: treat `H.MM` and `H:00` as times when the reference speaks a time.

### I-035 — Canary can emit runaway garbage
- On `en-it-05` (slow reading) Canary int8 produced "O P O N T H O R E N T E R E N T..." instead of text. Any product use needs an output sanity check (for example reject outputs made of isolated letters, or an out-of-range length versus the audio) and a fallback engine.

### I-036 — sherpa-onnx uses more cores than configured
- Median "busy cores" 7.4 (Whisper tiny, Canary) and 9.0 (Parakeet) with `num_threads = 4`; whisper.cpp stays at 3.5 to 3.8. Probably extra ONNX Runtime threads or spin-waiting (NOT verified). Matters because the desktop app and other programs share the CPU. To investigate: session options for intra/inter-op threads in sherpa-onnx.

### I-037 — whisper.cpp small is slower than real time on this CPU
- RTF 1.56 to 1.85, median 7 to 8 s for a sentence of about 5 s, p95 13 to 15 s. whisper.cpp large-v3-turbo (not installed) is heavier still. A GPU/Vulkan or Core ML build was not tested.

### I-038 — Benchmark limits
- One speaker, 95 sentences, 1 repetition, CPU only. No statistical claim beyond sampling noise over sentences. Confidence intervals come from `scripts/bootstrap_ci.py` (seed 7, 5000 resamples).

### I-018 update (2026-10-07)
- The repetition loops of sherpa-onnx Whisper tiny also occur on SHORT English questions (3 of 16 `en-general` outputs repeat the sentence 2 to 3 times), not only on long audio. They are not caused by `tail_paddings` (tested -1, 0, 50, 300, 1000). whisper.cpp tiny shows no such loop on the same files.

### I-039 — First full benchmark ran on a busy machine
- The owner started a heavy task during the run. Accuracy is expected to be unaffected (deterministic engines) but speed, memory and busy-core figures from `20261007-201137-full-owner-reps1` must not be quoted. A quiet-machine preflight now guards `bench run`. The preflight currently fails on this computer (CPU 16 to 52 % with Docker Desktop and the WSL VM active). Close: run the clean re-run, compare accuracy with the first run, replace the speed figures, update the PROJECT_LOG and D-032 (the provisional shortlist) if the speed picture changes.
- Related: I-012 (timing variance) stays open until a 3-repetition study is done on a quiet machine.

### I-039 update (2026-10-07) — resolved
- Clean re-run `20261007-215116-full-owner-reps1-clean`: 855 of 855 transcripts identical to the disturbed run, so accuracy is load-independent. Speed improved by 10 to 35 % (p95 of whisper.cpp small roughly halved); memory and busy cores unchanged. New figures are in the project log. Conditions: preflight 19.4 % CPU, AC power, limit relaxed to 30 % by D-034 (the laptop's background load never dropped below 16 %).

### I-012 update (2026-10-07)
- Still open: only 1 repetition. The preflight reading itself varied from 16 to 31 % between calls, so the load was not constant during the run; a 3-repetition study with the background load logged before and after is needed.

### I-036 update (2026-10-07)
- Confirmed on the clean run: sherpa-onnx busy cores 7.4 (Whisper tiny, Canary) and 9.4 (Parakeet) with 4 threads configured; whisper.cpp 3.5 to 3.9. Not caused by the disturbance. Cause still unknown.

### I-037 update (2026-10-07)
- Smaller than first measured but still true: whisper.cpp small q5_1 RTF 1.12 (greedy) and 1.24 (5 beams), median 5.4 to 5.9 s and p95 6.1 to 6.8 s for a sentence of about 5 s, at about 20 % background CPU. Still slower than real time on this CPU.


### I-012 update (2026-10-08) — measured, partly closed
- 3-repetition study `20261008-035938-timing-3reps` (40 short sentences, 9 configurations, 1080 runs, about 20 % background CPU): inside one run the spread of the same sample is small (median CV 0.7 to 1.4 % for whisper.cpp, 2.4 to 3.5 % for sherpa-onnx; per-repetition medians within 5 %), transcripts never change (0 unstable samples), and there is no cold-versus-warm inference difference for sherpa-onnx (the cost is the model load only).
- Still open: the BETWEEN-run difference (same samples, same settings, two different runs) reached 20 % for the fast models (Canary 463 -> 417 ms, Parakeet 517 -> 433 ms) and 0 % for whisper.cpp small, cause unknown; the background load was not logged continuously. Quote speed with a margin of about 10 to 20 % for fast models. A continuous load log in `bench` and a second independent run would narrow it down.

### I-036 update (2026-10-08)
- Reproduced in the timing study: busy cores 7.4 (Canary), 7.5 (Whisper tiny), 10.4 (Parakeet) with 4 threads configured; whisper.cpp 3.5 to 3.9. Still no cause.

### I-037 update (2026-10-08)
- Reproduced with variance: whisper.cpp small q5_1 RTF 1.26 (greedy) and 1.32 (5 beams); the whole min-max range of 40 sentences (5.0 to 6.7 s) lies above the sentence duration (about 4.4 s). Slower than real time is a stable property on this CPU, not noise.

### I-040 — Timing helper reports load 0 for sherpa-onnx
- `scripts/timing_study.py` prints the median `loadMs` per repetition, which is 0 for sherpa-onnx because the load is recorded on the first sample only. The cold load is the first record's `loadMs` (quoted by hand in the project log). Fixed the same day: the script now prints the median of the records that have a load and the maximum.

### Long-audio update (2026-10-08, M5e; D-035; run folders `20261008-*` in `benchmark/results/`)
- I-017 / I-019: chunking at 25 s does NOT cure Canary. On a 45.8 s private clip the closing word is still missing in both modes; chunked, Canary also returned half the words in 63 s (see I-041). Parakeet and every whisper.cpp model kept the ending, whole or chunked. Canary handled the three reference dictations (26 to 36 s) well as whole clips (1.5 % WER).
- I-018: sherpa-onnx Whisper cannot take more than 30 s (the library says so and discards the rest: measured on a 35.9 s dictation, last sentence lost). Chunking recovers the ending, but the repetition loops stay (private clip A: a 3-word group repeated 35 times whole, 28 times chunked; 15 s pieces produced one more loop on a dictation). The loop is a decoding defect of this runtime, not a length problem.
- I-029: measured now. Maximum clip length without chunking on this CPU and these clips: sherpa Whisper tiny 30 s (hard), Canary about 30 s to 36 s without loss on reference dictations but unreliable at 45 s, Parakeet and whisper.cpp no limit seen up to 45.8 s (3 reference dictations and 2 private clips only). Chunking at 25 s helps sherpa Whisper tiny (ending), is neutral for Parakeet and whisper.cpp, and hurts Canary. The planned comparison for the dataset is done; the private 1969 recording was not used.
- I-035: a second runaway of Canary (private clip, chunked). An output guard is still needed in any product: reject outputs whose length is far from the audio duration, or that took longer than a multiple of the audio length.

### I-041 — Canary int8 degrades on pieces of about 24 s
- Seen twice: (1) `en-dict-01` chunked at 25 s, first piece 0.3 to 24.3 s: one sentence dropped and one non-word, WER 3.7 % whole versus 22.2 % chunked; with 15 s or 10 s pieces the WER returns to 1.5 and 2.0 %. (2) private clip A (45.8 s), pieces 0 to 23.9 s and 23.9 to 45.8 s: 52 words instead of 104 and 63 s instead of 18 s. Not isolated (which piece, which tokens); a 20 s limit and a per-piece output guard are the obvious next experiments if Canary stays on the shortlist. Do not use `--chunk-max-s` 25 with Canary in a product.

### I-042 — Between-run speed offset larger than expected
- Clips of 25 s or less take the same path with or without chunking, yet the median inference time of `20261008-050621-short-vad` was 1 % (whisper.cpp base greedy) to 48 % (sherpa Whisper tiny) above the clean full run, although the measured background load before the run was lower (14.4 % versus 19.4 %). The preflight reading is a single 8 s average and varies by more than 10 points between calls. Speed figures from different runs must not be compared; compare inside one run only. Related: I-012.

### I-043 — Parakeet hotwords: surrogate vocabulary, lost elision, over-boosting
- Symptom: with the default character unit the library cannot encode any hotword ("Cannot find ID for token"); with the `bpe` unit it needs a `bpe.vocab` that the model download does not contain. A surrogate derived from `tokens.txt` (score = minus the id) works but the cut of a hotword into pieces may differ from the model's own.
- Measured (T4, 1 repetition, all 95 samples): score 1.5 fixes one drug name but writes "d amoxicilline" instead of "d'amoxicilline" (the hotword starts a new word) and breaks 4 words in the 36 s dictation; score 3.0 gives WER 5.1 % (baseline 2.3 %), 30 broken words, one runaway, vocabulary words inserted in English sentences.
- Status: open. To try: the real SentencePiece vocabulary (needs a download, ask the owner), scores between 1.5 and 3.0, per-word scores, a hotword form that keeps the apostrophe.

### I-044 — whisper.cpp initial prompt side effects
- Measured (T4): inference 25 to 40 % slower (tiny 762 -> 1070 ms, small 5881 -> 7261 ms median; between-run offset up to 20 %, I-042); six categories without key terms slightly worse (small 2.4 -> 3.0 %, base 6.0 -> 7.2 %, tiny 12.4 -> 13.8 %, 5 beams); tiny gets 51 to 54 broken words overall; small greedy lost a drug name. Words changed by the prompt include "jeudi" -> "jedi", "veuillez" -> "voyez".
- Status: open. Untested: prompt written as a sentence, `--carry-initial-prompt`, a shorter list.

### I-045 — Dictionary post-correction can rewrite correct words
- Mechanism: near-miss matching cannot tell a misspelling from a real word. Unit test `known_hazard_a_correct_look_alike_drug_name_is_replaced` shows "prednisolone" replaced by "prednisone" under the MEDIUM preset. The LOOSE preset turned "matin"/"mais" into "main" 63 times on the clean run.
- Mitigations in the code: strict default, minimum length, same first letter, plural guard, protected numbers/units/negations/dates, ambiguity left alone, every change returned. Not mitigated: a rare real word one edit from a vocabulary term under STRICT; any text outside the 95 sentences.
- Status: open until measured on free text and more speakers; the option stays OFF.

### I-046 — Import resampling and input formats
- `bench import` converts any PCM WAV to 16 kHz mono 16-bit. Clips at another rate go through the linear-interpolation resampler of `chunking.rs` (adequate for speech recognition input, not compared with a higher-quality resampler; the 11.025 kHz private clip of M5e was handled the same way). Compressed files (m4a, mp3, Ogg, FLAC) are refused with the existing explicit error. Workaround: convert to WAV first (Audacity or VLC).
- Status: open, low priority. To close: compare WER on one clip resampled both ways.

### I-047 — Private results contain the private texts
- Symptom: the "suspect samples" section of `summary.md` (benchmark.rs) quotes the reference and the best transcript of the hardest samples; `runs.jsonl` stores `reference` and `text` of every run. For `bench run --include-private` these files live in `benchmark/results/private/<run>/` (git-ignored).
- Risk: copying a figure table is fine; copying that section or any line of `runs.jsonl` into a document, a chat or a commit would leak a third party's words. Also, `bench run --include-private` without `--category` mixes private and public samples in one folder.
- Workaround: always pass `--category <private category>`; report counts and WER only. Possible fix (not done): omit the texts of private samples from `summary.md`, which needs a `private` flag on the run record.
- Status: open.

### I-048 — Kokoro int8 is slower than real time on this CPU
- Measured (M6, run `20261008-085403-tts-m6`): RTF 1.55 to 2.06 for French and English voices (3.4 s to generate 1.6 s of speech, 18.3 s for 10.4 s), 4 threads, int8 weights, background load 17 to 33 %. An earlier unmeasured smoke test gave 1.1 to 1.2 (between-run offset, see I-042). Piper is 13 to 29 times faster than real time on the same machine.
- Impact: Kokoro cannot be used for interactive replies on this class of CPU without a wait; fine for offline generation.
- Status: open. To try: fp32 weights, other thread counts, a shorter text split with playback starting on the first chunk (streaming), a smaller model.

### I-049 — Voice metadata is thin
- Piper cards do not document the speakers' gender (shown as "unknown", never guessed); `libritts_r` has 904 speakers and the app lists only the first 12 (any number can be used through the voice id); the Kokoro voice table gives gender but only one French voice exists.
- Status: open (information). A French male voice of acceptable quality and commercial licence is still not identified (the question of M0 section 5); candidates not examined: `fr_FR-upmc` (CC-BY-SA, 2 speakers), the voices with unknown licence.

### I-050 — Kokoro French logs "Skip unknown phonemes" for a hyphen
- The library printed `Skip unknown phonemes. Unicode codepoint: U+002D` while synthesising a French sentence that contains no hyphen. Cause not isolated (probably a phoneme produced by the phonemizer); audible effect not judged. Open until the owner has listened to the Kokoro French samples.

### I-051 — TTS needs the libraries that contain the GPL phonemizer
- Any application build that synthesises speech must link the full sherpa-onnx libraries (espeak-ng, GPL-3.0). The "no-tts" libraries avoid it but cannot synthesise. Extends I-003, I-010 and D-012. Status: open, owner/legal decision before any distribution of TTS (M8).

### I-048 update (2026-10-08, second run) — Kokoro speed varies a lot between runs
- Run `20261008-094413-tts-m6-licence-focus` (8 voices in one run, preflight 23 %): Kokoro int8 RTF 1.10 to 1.33; the first M6 run gave 1.55 to 2.06 for the same voices. Statement to use: "slower than real time here, by 1.1 to 2.1 depending on the run". Piper and Coqui stayed at 0.035 to 0.064 in both runs.

### I-052 — Fine-tuned Piper voices inherit an unclear licence
- Piper `fr_FR-siwis` is fine-tuned from the English Lessac voice (data under a Blizzard 2013 research licence agreement, granted to a named person or organisation; clauses on derived models not read); `fr_FR-gilles-low` from the English Ryan voice (data CC BY-NC-SA 4.0) although its own dataset is CC0. Whether the weights inherit the restriction is a legal question.
- Status: open. Ratings: siwis `review`, gilles `excluded`. Workarounds: use voices trained from scratch (ljspeech, kristin, libritts_r for English; Piper `mls` for French after conversion; Coqui css10).

### I-053 — Kokoro training audio includes output of closed TTS services
- The model card lists "synthetic audio generated by closed TTS models from large providers" among the training data; those providers' terms were not read; plus SIWIS and Koniwa data that need attribution. Weights are Apache-2.0. Status: open, rating `review`.

### I-054 — The GPL phonemizer code stays in the library even for a character-based voice
- The Coqui voice never calls espeak-ng and has no phonemizer data, but the TTS-capable sherpa-onnx static libraries are built with it. Whether a distributed binary that contains but never calls it is a licence problem is a legal question. Technical ways out, not tried: a sherpa-onnx build without the phonemizer, or another runtime for character-based VITS. Extends I-051 and D-012. Status: open.

### I-055 — Character-based voices need a text normaliser
- Measured: the Coqui French voice does not speak digits ("12 mars à 14 h 30" came out as "mars ..." with the numbers missing). Every phonemizer-based voice spoke them. A product using this voice must expand numbers, dates, times, units and abbreviations before synthesis (French and English). `numbers.rs` handles the opposite direction (spoken words to digits) for scoring only. Status: open, not built.

### I-056 — Synthesis is not deterministic
- The same text regenerated gives slightly different audio (sampling noise in the voice model) and therefore different recognised words (for example the same drug name heard four different ways across two generations). Any measurement that depends on the exact audio (recogniser round trips) needs repetitions; the speed figures are unaffected in kind but vary between runs (I-042, I-048).

### I-057 — Audible hiss on the Kokoro voices
- Reported by the owner in a blind listening (2026-10-08): "sifflement ou grésillement" on all three Kokoro voices (French `ff_siwis`, English `af_heart` and `am_adam`), absent on every Piper and Coqui voice. No measurement of mine detected it. Candidate causes, none verified: int8 quantisation of the package used, the 24 kHz output, the vocoder. Not tried: the fp32 Kokoro package (about 330 MB download, needs the owner's approval). Status: open, low priority since Kokoro is deprioritised (D-041).

### I-055 update (2026-10-08)
- The owner confirmed by ear that the Coqui voice does not speak the digits (12, 14 and 30) and that "Rendez-vous" is mispronounced. Decision D-041: build the normaliser (numbers, dates, times, units, abbreviations) for character-based voices. Status: accepted for implementation.

### I-055 update (2026-10-08, M6d) — mitigated
- The normaliser is built (D-042, `normalise.rs`, 15 unit tests, French and English) and on by default for the Coqui voice through the manifest flag `normalizeText`. Rough check with the speech recogniser (Parakeet, six French sentences with numbers, dates, times, units, a fraction and a phone number, three repetitions, two complete runs): key numbers heard 0/36 without it, 28/36 and 30/36 with it. The owner's ear is the judge (listening session 2). Remaining defects are listed in I-058 and I-061. Status: mitigated; close after the owner's listening.

### I-058 — The Coqui voice stays weak even when it speaks the numbers
- With the normaliser on, the recogniser still mangled the date sentence ("Rendez-vous le 12 mars à 9h00" heard as "J'ai voulu doucement à 9 heures" and "Rendez-je vous les douze morts à 9 heures"), the spoken text being correct: the voice's own articulation of some French words is poor (the owner already scored clarity 3 and accent 2, and heard "Rendez-vous" mispronounced). A long phone number came back as "06 1234 56 78". Status: open. Next step: the owner's session 2 compares it with three speakers of the converted `mls` voice; no better freely usable French voice is installed yet.

### I-059 — The application has no dark theme
- Only the new read-along block follows the system setting (`prefers-color-scheme`, own text and background colours, contrast 12.7 light and 8.5 dark measured). The rest of the page keeps fixed light colours, so in dark mode the block is a dark panel on a light page. Not a defect of the highlight; a full theme is outside this lab's scope unless the owner asks. Status: open, low priority.

### I-060 — `mls` is a weak candidate and cannot be installed from the app
- The Piper `fr_FR-mls-medium` voice (CC BY 4.0, trained from scratch, 125 speakers) works in sherpa-onnx after a metadata conversion, but by a recogniser proxy 74 of 125 speakers have a word error rate of 90 % or more, only 18 reach 50 % or better and the best three 33 % (references: Coqui 6.7 %, siwis 6.7 %, gilles 20 %). Naturalness is not measured. Because the conversion needs Python, the voice has the packaging kind `local`: the app's Install button refuses it with the script name. A product would need the converted package hosted somewhere or the conversion done at build time. Status: open, waits for the owner's ear (session 2).

### I-061 — Known limits of the text normaliser
- Acronyms (IRM, ECG, AVC) are not spelled; the feminine "une" is used only after a short list of common nouns; Roman numerals (XIXe) are not converted; "10.30" is a time only after a time word (otherwise a decimal), and "version 2.15" style numbers are decimals; an invoice number after "numéro" is read as an ordinary number (up to 12 digits, no leading zero); numbers separated by single spaces ("5 100") are read as one number; a standalone capital letter followed by a dot is taken as an initial by the splitter; English sentences are normalised by the same unit tests but no character-based English voice exists to hear them; Swiss variants (septante, huitante, nonante) are not produced (the owner's rule asked for soixante-dix and quatre-vingts). Status: open, extend on real texts.

### I-062 — Sentence segmentation is a heuristic
- A line break inside a hard-wrapped paragraph splits the sentence; a sentence that really ends before a lower-case word ("... Bonjour. merci") is not split; the abbreviation list is short and fixed. Wrong splits only change where pauses and shading fall, never the text spoken. Status: open, low priority.

### I-063 — The good French voices have a restricted lineage (owner's session 2 and interface tests)
- Owner's reading (2026-10-08): best French voices Piper siwis (female) and gilles (male), all other French voices unsatisfactory (Coqui css10 with the normaliser: 3 out of 5, converted `mls`: 1 to 2). siwis = `review` (parent Lessac, Blizzard 2013 research licence), gilles = `excluded` (parent Ryan, CC BY-NC-SA 4.0): see I-052. No other French voice of the sherpa-onnx release qualifies (tom, tjiho: AGPL-3.0; upmc: BY-SA from Lessac; miro: non-commercial; mms: non-commercial).
- Status: open. Proposed way out: re-train both voices from the clean public base checkpoint (D-043); interim for a beta: English voices plus Coqui for French. Related: I-052, I-060, I-058.

### I-063 update (2026-10-08) — owner's decision
- Decision D-043: keep Piper siwis and gilles now; re-train both from the clean base checkpoint later, when the software is to be commercialised. Until then they are for development, evaluation and non-commercial use; their ratings stay `review` and `excluded`. Status: open as a commercial-release blocker; nothing was re-trained, nothing about the legal question was resolved. Still to confirm: whether handing a free beta to other people counts as non-commercial use (a legal question).

### I-064 — Building the package needs the staged sidecar (M7)
- Symptom: after M7 any `cargo build`, `cargo test` or `cargo run --example` of the `speechlab` package fails with "resource path `binaries\whisper-cli-x86_64-pc-windows-msvc.exe` doesn't exist" when `src-tauri/binaries/` is missing (VERIFIED by moving the folder away and running `cargo check`).
- Cause: `tauri-build` checks every `externalBin` and `resources` entry of `tauri.conf.json` at build time.
- Workaround: run `scripts/build-whisper-cpp.ps1` once, then `scripts/stage-whisper-sidecar.ps1`; the staged folder is git-ignored. A fresh clone or a CI machine needs both. Status: open (documented in README and HANDOFF).

### I-065 — A cancelled transcription is shown as an error (M7)
- Symptom: in the packaged app, pressing Cancel during a whisper.cpp transcription stops the `whisper-cli.exe` process correctly (VERIFIED) but the red banner then reads "operation cancelled". The TTS panel ignores that message (`TtsPanel.tsx`: "A cancellation the user asked for is not an error"), the transcription code in `App.tsx` does not.
- Status: open, not fixed (the owner approved only the CSP change in M7). Proposed fix: the same filter in `App.tsx`. The packaged-app UI test (`scripts/ui_test_packaged.mjs`) has a check for it, which fails today (23 of 24).

### I-066 — The web view runtime keeps a connection to a Microsoft address while the app runs (M7)
- Observed (offline proof): with a firewall rule that blocks all outbound traffic of `speechlab.exe` and `whisper-cli.exe`, the app's own processes made no connection (a download attempt from the app failed within 0.2 s with "os error 10013"), but the WebView2 network service process (`msedgewebview2.exe --type=utility --utility-sub-type=network.mojom.NetworkService`, part of the system runtime, not of the application) held one established connection to `2620:1ec:33::11` port 443 (a Microsoft address range). Only that one outside connection was seen in 88 samples over about 40 s, plus the loopback debugging port that the test itself opened.
- Meaning: "offline" is proven for the application's code (speech engines, model loading), not for the system web view. Purpose of the connection NOT VERIFIED (possibly the runtime's own update, reputation or telemetry service). Not tried: starting the web view with `additionalBrowserArgs` that disable background networking, or blocking the runtime in a firewall (the runtime is shared with other applications, so it was not touched). Status: open; matters for the privacy statement of M8.

### I-067 — No credits view in the app and no licence page in the installer (M7)
- Observed: the model and voice tables show each item's licence text and each voice's commercial-use rating (VERIFIED in the packaged app), and a warning is shown for `review` and `excluded` voices, but nothing lists the credits that CC BY 4.0 and BSD-3-Clause require, the GPL-3.0 notice for espeak-ng, or the MIT notices; the NSIS installer shows no licence page.
- Status: open, proposed (an "About and credits" section generated from the manifest, and an installer licence file). The owner did not choose it in M7; the content (what the owner is willing to state) is theirs to validate. Needed before any distribution.

### I-068 — Unsigned installer and executables (M7)
- Observed here (VERIFIED): the unsigned installer (`Get-AuthenticodeSignature`: NotSigned) installed silently in about 2 s and the unsigned `speechlab.exe` and `whisper-cli.exe` ran; Avast (the active antivirus; Defender's real-time protection is off while Avast is registered) blocked or quarantined nothing, and Defender recorded no detection mentioning the files. Avast's own log was not read.
- NOT VERIFIED: what SmartScreen shows to a person who downloads the installer with a browser (the installer was started from a script, so the "mark of the web" path was never exercised); how other antivirus products react. Code signing would need a certificate from a certificate authority (an organisation-validation or extended-validation certificate, paid, annual; a cloud signing service is the other route) and the signing step added to the build; SmartScreen reputation builds with downloads even for signed files. Prices and rules NOT VERIFIED (not read from a vendor). Status: open.

### I-069 — Installer side effects (M7)
- VERIFIED: the installer creates a Desktop shortcut and a Start Menu shortcut by default, and the registry keys `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\SpeechLab` and `HKCU\Software\assistantcabinet\SpeechLab`. The silent uninstaller removed the install folder and the uninstall key, kept the user data (tested with dummy data folders while the real ones were renamed away and restored: 2580 and 742 files before and after) but left the `HKCU\Software\assistantcabinet\SpeechLab` key. The interactive uninstaller, which shows a choice about deleting the application data, was NOT tested; its default and its effect on the models folder are unknown. Status: open, minor.

### I-070 — The whisper.cpp sidecar needs the Visual C++ runtime, which the installer does not carry (M7)
- Observed (VERIFIED by reading the import tables of the files in `src-tauri/binaries/`): `speechlab.exe` imports only the universal C runtime (`api-ms-win-crt-*`, part of Windows 10 and later; sherpa-onnx is linked with the static MT runtime), but `whisper-cli.exe`, `whisper.dll`, `ggml.dll`, `ggml-base.dll` and `ggml-cpu.dll` import `MSVCP140.dll`, `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll` and (the two ggml libraries) `VCOMP140.dll`. This machine has them (Visual Studio Build Tools).
- Consequence on a clean Windows without the "Microsoft Visual C++ 2015-2022 Redistributable": the application, the sherpa-onnx engines and the voices should start, but every whisper.cpp transcription would fail because the sidecar cannot start. NOT VERIFIED (no clean machine or virtual machine available; Windows Sandbox or a clean VM would do).
- Way out, not done: rebuild whisper.cpp with the static runtime and static libraries (`-DBUILD_SHARED_LIBS=OFF` and `-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded` in `scripts/build-whisper-cpp.ps1`): one executable, no four libraries, no redistributable. The alternative is to install the redistributable from the setup program. Status: open; recommended before any distribution.

### I-071 — Control characters in README.md (found and fixed in M8)
- Symptom: the sentence about the recordings folder read `%APPDATA%<bell>i.assistantcabinet.speechlab<line break>ecordings`. Cause: in an earlier edit the sequences `\a` and `\r` of the Windows path were interpreted as escape codes by the editing tool. Fix: the path is now `%APPDATA%\ai.assistantcabinet.speechlab\recordings`; the file contains no control character other than line breaks (checked by a byte scan). Trap for future edits: write backslashes with care when a tool or a script processes the text (in a script use `chr(92)`). Status: resolved.

### I-072 — The SpeechLab code has no licence file
- Observed in M8: there is no `LICENSE` or `NOTICE` in the repository; `cargo metadata` reports no licence for the `speechlab` package. Any code moved into AssistantCabinetAI, or published, needs a licence decision (the owner's), and the third-party notices (CC BY 4.0, BSD-3-Clause, MIT, Apache-2.0, MPL-2.0, GPL-3.0 for espeak-ng) need a place (I-067). Status: open.

### I-073 — Licence readings still missing (M8)
- Not read: the training-data provenance of Parakeet and Canary (cards read only for their licence, CC BY 4.0, in M0); the redistribution terms of the Visual C++ runtime DLLs imported by the sidecar, of the WebView2 bootstrapper and of the installer builder; the five MPL-2.0 crates (cssparser, cssparser-macros, dtoa-short, option-ext, selectors; declared field only); the discrepancy between the manifest (Whisper weights MIT) and one model card (Apache-2.0), noted in M0 and never reconciled; the licence file of ggml inside the whisper.cpp repository. Listed for the lawyer in report section 9.6. Status: open.

### I-074 — Evidence gaps behind the provisional recommendation (M8)
- Never run: whisper.cpp `large-v3-turbo` (in the manifest, 574 MB, not installed), any GPU or accelerated build (Vulkan, Metal, Core ML), a second or weaker machine (Parakeet needs 1.6 to 1.9 GB and 7 to 10 busy cores), speech with noise or conversational style, speaking-speed variations, other speakers and microphones, streaming recognition (the French streaming models' licences are unknown, I-004). The recommendation D-047 names the Parakeet margin over whisper.cpp small as borderline (+1.6 points, interval +0.1 to +3.1) and says what would settle each point (report section 10.3). Status: open.
