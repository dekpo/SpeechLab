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
| I-017 | Open | engine | Canary logs "first generated token is <|endoftext|> ... (issue #3919)" on a 45.8 s recording; output was still produced |
| I-016 | Explained (partly) | evaluation | Owner-reported: Whisper tiny transcribed the owner's recording very badly. Cause for sherpa-onnx's Whisper tiny: repetition loop on a 45.8 s file (see I-018); not a verdict on whisper.cpp |
| I-018 | Open | engine | sherpa-onnx Whisper tiny (greedy) falls into a repetition loop on a recording longer than 30 s |
| I-019 | Confirmed | evaluation | Canary int8 dropped the final spoken sentence of the 45.8 s owner recording (owner listened: it is spoken) |
| I-020 | Open | design | whisper.cpp as external process = cold start on every run |
| I-021 | Open | packaging | `whisper-cli` is located in `vendor/` in dev only; must become a bundled sidecar (M7) |
| I-022 | Mitigated | environment | `vswhere.exe` missing: Visual Studio CMake generator unusable; NMake used instead |
| I-028 | Mitigated | audio input | Owner's microphone input seems to saturate (recording quality mediocre); a level/clipping indicator was added, cause not confirmed |
| I-029 | Open | engines | Long audio (45 s private clip, 1969 speech MP3): Canary truncates, Whisper tiny (both runtimes) repeats near the end; chunking/VAD needed (see I-018, I-019) |
| I-031 | Mitigated | git hosting | GitHub returned HTTP 500 when creating any new branch pointing at the M4 commit; creating the branch from main in the web UI worked |
| I-032 | Open | ui | The dataset recorder shows the benchmark path with a ".." segment (cosmetic) |
| I-039 | Resolved | benchmark | First full run was disturbed by a heavy task; clean re-run done (855/855 transcripts identical, new speed figures at about 20 % background CPU) |
| I-033 | Open | dataset | Suspect sample en-it-05: all engines hear 543, the script says 443 (listen, maybe re-record) |
| I-034 | Open | scoring | Detector false alarms on times written "10.30", "3.30", "9h00" (about 15 % of reviewed flags) |
| I-035 | Open | engines | Canary int8 emitted runaway garbage on one slow recording; product needs an output guard |
| I-036 | Open | performance | sherpa-onnx uses 7 to 9 busy cores with 4 threads configured |
| I-037 | Open | performance | whisper.cpp small is slower than real time on this CPU (RTF 1.6 to 1.9) |
| I-038 | Info | benchmark | One speaker, 95 sentences, 1 repetition, CPU only; sampling interval via bootstrap |
| I-030 | Open | scoring | Swiss number words (septante, huitante, nonante) are not folded to digits |
| I-024 | Open | audio input | Microphone: WebView2 permission prompt on first use; persistence across restarts, release-build origin and macOS behaviour unverified |
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
