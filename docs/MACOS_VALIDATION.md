# macOS validation checklist (Apple Silicon and Intel)

**ON HOLD until further notice (owner, 2026-10-09, D-048, I-075): the owner has no Mac available to implement and test a macOS version. This checklist is kept, not cancelled; no macOS claim may be made.**

Status: **everything in this file is NOT VERIFIED.** No Mac was available during M7 (2026-10-08). The file
lists what to check, in order, on a real Mac, with what to record. Items marked *(read)* rest on a file
that was read in this repository or in a dependency's source; they are still not observed on a Mac.
Where a line comes from general knowledge of the platform and not from a file, it says *(knowledge)*:
re-check it against the current vendor documentation before relying on it.

Windows reference results (M7): see `docs/PROJECT_LOG.md` entry "M7 packaging". Every test below has a Windows
counterpart so that the two platforms can be compared line by line.

## 0. What to record for every line

`PASS`, `FAIL` or `SKIPPED`, the exact command, the output or screenshot name, the macOS version, the machine
(model, chip, RAM), and the date. A failure is as valuable as a pass: write it in `docs/ISSUES.md`.

## 1. Prerequisites (NOT VERIFIED)

| # | Step | Expected |
|---|---|---|
| 1.1 | macOS version and chip: `sw_vers`, `uname -m` | note them; Apple Silicon = `arm64`, Intel = `x86_64` |
| 1.2 | Xcode command line tools: `xcode-select --install`, then `clang --version` | installed |
| 1.3 | Rust (stable) with both targets: `rustup target add aarch64-apple-darwin x86_64-apple-darwin` | both listed by `rustup target list --installed` |
| 1.4 | Node.js, `corepack enable`, `pnpm install` | no error |
| 1.5 | CMake (`brew install cmake`) for whisper.cpp | `cmake --version` |
| 1.6 | `pnpm typecheck && pnpm test && pnpm build` | same results as on Windows (16 vitest tests) |
| 1.7 | `cd src-tauri && cargo test --lib` | 163 tests; two use the real Piper siwis voice if installed, the others do not need a model. Note any test that fails only on macOS |

## 2. sherpa-onnx libraries (NOT VERIFIED)

The Windows build links the static MSVC libraries downloaded by `scripts/fetch-sherpa-libs.ps1`
(because an antivirus intercepts TLS on the Windows machine, I-009). The crate `sherpa-onnx-sys` 1.13.8
picks the archive from the target (*read*, `build.rs`, function `archive_name`):

| Target | Static archive the crate looks for |
|---|---|
| `aarch64-apple-darwin` | `sherpa-onnx-v1.13.8-osx-arm64-static-lib.tar.bz2` |
| `x86_64-apple-darwin` | `sherpa-onnx-v1.13.8-osx-x64-static-lib.tar.bz2` |

| # | Step | Expected |
|---|---|---|
| 2.1 | Let the crate download the archive on its own (first `cargo build`). If a proxy breaks it (`UnknownIssuer`), download the archive with `curl -L` from the release page of sherpa-onnx v1.13.8, check its SHA-256 against the value on the release page, put it in a folder and set `SHERPA_ONNX_ARCHIVE_DIR` (*read*: the crate honours this variable, and `SHERPA_ONNX_LIB_DIR` for an already extracted folder) | build passes |
| 2.2 | A shell equivalent of `scripts/fetch-sherpa-libs.ps1` does not exist yet. Write `scripts/fetch-sherpa-libs.sh` only if step 2.1 needed the manual path | optional |
| 2.3 | Confirm the archive contains the espeak-ng phonemizer code (it does on Windows, I-051): `strings` on the library, or the file list. The GPL-3.0 obligation of D-012 / I-051 applies to the macOS build exactly as to the Windows one | same finding as Windows |
| 2.4 | Link warnings about the C++ runtime or Accelerate/CoreFoundation frameworks *(knowledge)*: record them | none, or recorded |

## 3. whisper.cpp command line tool (NOT VERIFIED)

Windows: `scripts/build-whisper-cpp.ps1` builds tag `v1.9.4` with CPU only, as shared libraries; the
sidecar needs `whisper-cli.exe` plus four DLLs (VERIFIED on Windows: without any one of them it exits 127).
On macOS prefer a **static** build so that the sidecar is one file and no `@rpath` problem can occur.

| # | Step | Expected |
|---|---|---|
| 3.1 | `git clone --depth 1 --branch v1.9.4 https://github.com/ggml-org/whisper.cpp.git vendor/whisper.cpp` | cloned |
| 3.2 | Apple Silicon, Metal: `cmake -S vendor/whisper.cpp -B vendor/whisper.cpp/build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DGGML_METAL=ON -DGGML_METAL_EMBED_LIBRARY=ON -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF` then `cmake --build vendor/whisper.cpp/build --config Release -j` *(knowledge: option names must be checked in the v1.9.4 CMake files)* | `build/bin/whisper-cli` exists |
| 3.3 | Intel: same with `-DCMAKE_OSX_ARCHITECTURES=x86_64` and Metal left off unless measured useful | `file build/bin/whisper-cli` says `x86_64` |
| 3.4 | Core ML is a separate option (`-DWHISPER_COREML=1`) and needs a generated Core ML encoder next to each ggml model *(knowledge)*: it cannot be served by the model manager as it is. Treat it as a later experiment, not a packaging requirement. Record only whether Metal alone is faster than CPU | decision recorded |
| 3.5 | `otool -L build/bin/whisper-cli` | no dependency outside system libraries and frameworks |
| 3.6 | Transcribe the same short clip with CPU and with Metal builds | transcripts identical or the differences recorded: the Windows accuracy figures (benchmark) are CPU figures and **do not transfer** to a Metal build without a new benchmark run |
| 3.7 | Sidecar file names for Tauri's `externalBin`: `src-tauri/binaries/whisper-cli-aarch64-apple-darwin` and `src-tauri/binaries/whisper-cli-x86_64-apple-darwin` (the suffix is the Rust target triple, as `whisper-cli-x86_64-pc-windows-msvc.exe` on Windows). For a universal bundle Tauri may need `whisper-cli-universal-apple-darwin` *(knowledge)*: check, or build two separate installers | files in place; `pnpm tauri build` accepts them |
| 3.8 | `scripts/stage-whisper-sidecar.ps1` is Windows only; a `.sh` equivalent must be written (copy + rename + `chmod +x`) | optional script |

## 4. Build the application (NOT VERIFIED)

| # | Step | Expected |
|---|---|---|
| 4.1 | `pnpm tauri build --target aarch64-apple-darwin` (on an Apple Silicon Mac) | `.app` and `.dmg` under `src-tauri/target/aarch64-apple-darwin/release/bundle/` |
| 4.2 | `pnpm tauri build --target x86_64-apple-darwin` (on either Mac; on Apple Silicon it needs Rosetta for the test run) | same under `x86_64-apple-darwin` |
| 4.3 | Optional `--target universal-apple-darwin`: needs both sherpa archives and both sidecars | record success or the error |
| 4.4 | `tauri.conf.json` today lists `"targets": ["nsis"]`: add `"dmg"` / `"app"` for macOS (a decision to record in `docs/DECISIONS.md`); `bundle.icon` already includes `icons/icon.icns` (*read*) | config accepted |
| 4.5 | Record: build time, bundle size, `du -sh` of the `.app`, `file` of the main binary, `otool -L` of the main binary and of the sidecar | numbers, not impressions |
| 4.6 | If the sidecar's extra shared libraries are not avoidable (3.5), list them in `bundle.macOS.frameworks` or as resources | no missing-library error at launch |

## 5. WKWebView differences (NOT VERIFIED)

The app runs in the system web view: WebView2 (Chromium) on Windows, WKWebView (Safari engine) on macOS.

| # | Step | Expected |
|---|---|---|
| 5.1 | Microphone text: `NSMicrophoneUsageDescription` must exist in the application's `Info.plist` (Tauri merges a `src-tauri/Info.plist`) *(knowledge)*. Add a sentence in plain language | the system prompt shows the sentence |
| 5.2 | Hardened runtime (needed for notarisation) requires the entitlement `com.apple.security.device.audio-input` *(knowledge)*; put it in a `src-tauri/entitlements.plist` and reference it from `bundle.macOS.entitlements` | microphone capture works in the signed build, not only in `pnpm tauri dev` |
| 5.3 | Microphone permission in the release origin: grant it, quit, relaunch. Does the prompt come back every time? (Windows result is in the M7 log entry; I-024 stays open for macOS) | persistent or not, recorded |
| 5.4 | `getUserMedia` and `AudioWorklet` (`src/audio/recorder.ts`): record 5 s, save the clip, play it back | clip saved, correct duration |
| 5.5 | Audio import by decoding in the web view (`src/audio/wav.ts`, I-026): import one WAV, one MP3, one M4A (AAC), one Ogg Vorbis, one Ogg Opus | WAV, MP3, M4A expected to work; **Ogg support in WKWebView is uncertain** *(knowledge)*: record each result. If Ogg fails, the import must say so clearly (today it relies on the web view's decoders) |
| 5.6 | Audio playback of the generated WAV, read-along highlight driven by the player's clock (`src/components/ReadAlong.tsx`): muted playback, check the highlight moves | highlight follows; autoplay policy: nothing plays by itself (project rule) |
| 5.7 | Dark/light: the page has no dark theme (I-059); the read-along block follows the system setting | same as Windows |
| 5.8 | CSS and layout: open the three tabs at 960x720 and at a small window | no clipped controls |
| 5.9 | UI automation: the Chrome DevTools protocol script (`scripts/ui_test_readalong.mjs`) does **not** work on WKWebView. Use the Safari Web Inspector (Develop menu, enable "Show features for web developers") by hand, or write the same 33 checks as a manual list | manual results, line by line |

## 6. Where files live (NOT VERIFIED)

| # | Item | Expected on macOS *(knowledge)* | Check |
|---|---|---|---|
| 6.1 | Models folder | `~/Library/Application Support/ai.assistantcabinet.speechlab/models` (the app uses Tauri's `app_data_dir()`, *read* `lib.rs`; override `SPEECHLAB_MODELS_DIR`) | `ls` after installing a model |
| 6.2 | Recordings and generated speech | same folder, `recordings` and `tts-output` | `ls` |
| 6.3 | espeak-ng data | inside each voice package in the models folder (`espeak-ng-data`), downloaded, not bundled | a Piper voice generates a WAV from the installed `.app` |
| 6.4 | Sidecar location | `SpeechLab.app/Contents/MacOS/whisper-cli` next to the main executable (the code looks next to `current_exe()` first, *read* `whisper_cpp.rs`) | `ls SpeechLab.app/Contents/MacOS` |
| 6.5 | Model download over TLS | native certificates (`ureq` with `native-certs`) | a model downloads; behind an interception proxy, record the behaviour |

## 7. Signing, hardened runtime, notarisation (NOT VERIFIED)

| # | Step | Expected |
|---|---|---|
| 7.1 | Apple Developer Program membership (paid, annual) and a "Developer ID Application" certificate *(knowledge)*; price and rules to confirm on Apple's page | certificate in the keychain |
| 7.2 | Sign the app **and the sidecar** with the hardened runtime: `codesign --force --options runtime --timestamp --entitlements entitlements.plist --sign "Developer ID Application: ..." <path>`; Tauri can do it from `bundle.macOS.signingIdentity` or environment variables *(knowledge)* | `codesign --verify --deep --strict --verbose=2 SpeechLab.app` passes |
| 7.3 | Check whether ONNX Runtime inside sherpa-onnx needs extra entitlements under the hardened runtime (JIT or unsigned executable memory) *(knowledge)*: only add what a failing test proves necessary | app transcribes and synthesises when signed |
| 7.4 | Notarise: `xcrun notarytool submit SpeechLab.dmg --apple-id ... --team-id ... --wait`, then `xcrun stapler staple SpeechLab.dmg` | status Accepted; stapled |
| 7.5 | Gatekeeper: copy the `.dmg` through a download (quarantine flag), open it, drag the app, launch. `spctl --assess --type execute -vv SpeechLab.app` | accepted |
| 7.6 | Unsigned build behaviour (what a tester without the certificate sees): quarantine dialog, "right-click > Open" path, `xattr -dr com.apple.quarantine` | recorded |
| 7.7 | App Sandbox (Mac App Store) is out of scope: the sidecar process, the models folder and a free file picker would all need entitlements | not attempted |

## 8. Functional checks in the installed app (NOT VERIFIED)

The same list as the Windows installed-app checks (M7), in this order:

1. The window opens; the model list shows; with no model installed the app says so and offers the install buttons (no crash).
2. Install `sherpa-whisper-tiny` (or the smallest model) from the app; transcribe a short clip.
3. whisper.cpp engine through the sidecar: transcribe the same clip.
4. Microphone: prompt text, choice persistence (5.3).
5. A Piper voice generates a WAV; the Coqui voice speaks numbers (normaliser on); the read-along highlight moves during a muted playback; Cancel works.
6. Offline proof: switch Wi-Fi off, repeat one transcription and one synthesis.
7. Idle memory (Activity Monitor or `ps -o rss`), time to a visible window (stopwatch or screen recording, say which).
8. Quit, relaunch, check that settings, clips and models persist.

## 9. Licence and credit points that apply on macOS too

- espeak-ng (GPL-3.0) is in the sherpa-onnx libraries of every build that synthesises speech (D-012, I-051, I-054).
- Credits for voices rated `attribution` (CC BY 4.0, BSD-3-Clause): the app and the installer need a credits place (see the M7 log entry and the open item recorded there).
- The two French Piper voices with a restricted parent checkpoint are development-only until re-trained (D-043, I-063): not a default in any commercial build.
- Models are never bundled in the `.dmg`.

## 10. Result table (fill in on the Mac)

| Section | Apple Silicon | Intel | Notes |
|---|---|---|---|
| 1 prerequisites | NOT VERIFIED | NOT VERIFIED | |
| 2 sherpa-onnx libraries | NOT VERIFIED | NOT VERIFIED | |
| 3 whisper.cpp sidecar | NOT VERIFIED | NOT VERIFIED | |
| 4 build | NOT VERIFIED | NOT VERIFIED | |
| 5 web view | NOT VERIFIED | NOT VERIFIED | |
| 6 file locations | NOT VERIFIED | NOT VERIFIED | |
| 7 signing and notarisation | NOT VERIFIED | NOT VERIFIED | |
| 8 functional checks | NOT VERIFIED | NOT VERIFIED | |
