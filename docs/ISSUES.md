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
