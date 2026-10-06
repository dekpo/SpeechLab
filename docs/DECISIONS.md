# Decisions

One entry per non-trivial choice. Never delete a superseded decision; mark it `Superseded by D-xxx`.

---

## D-001 — Tauri 2.x stable, not Tauri 3 alpha
- Date: 2026-10-06 · Status: accepted
- Context: Tauri 2.12.1 is stable; 3.0.0-alpha.4 exists. The plan specifies Tauri 2.
- Decision: pin Tauri 2.x. Ignore 3.x alpha.
- Consequences: re-evaluate only when 3.x is stable and the owner asks.

## D-002 — Start with sherpa-onnx; whisper.cpp afterwards
- Date: 2026-10-06 · Status: accepted (owner decision)
- Context: `Plan.md` suggests M2 = whisper.cpp, M3 = sherpa-onnx. sherpa-onnx has prebuilt libraries and an official Rust crate (no CMake/LLVM needed), and can also run Whisper models.
- Decision: swap the order: **M2 = sherpa-onnx STT, M3 = whisper.cpp**. Milestone content is unchanged; this is a re-ordering, not a skip.
- Consequences: CMake and LLVM are not needed until M3. The provider contract is designed in M1 so both orders work.

## D-003 — Free and open-source first
- Date: 2026-10-06 · Status: accepted (owner decision)
- Decision: prefer free/open-source tools, libraries, and models. Any exception must be listed here with justification.
- Known exception candidate: **MSVC Build Tools** (required by Tauri on Windows) is free to download but is not open source and has its own license terms. Alternative toolchains are not officially supported by Tauri. Owner to confirm acceptance.
- Free/open tools in use: Node.js (MIT), Rust (MIT/Apache-2.0), Tauri (MIT/Apache-2.0), React (MIT), Vite (MIT), pnpm (MIT), Git (GPL-2.0, as a tool only), CMake (BSD-3) and LLVM (Apache-2.0 with LLVM exception) when needed.

## D-004 — espeak-ng (GPL-3.0) is a flagged licensing risk
- Date: 2026-10-06 · Status: open
- Context: sherpa-onnx's Piper/VITS/Kokoro TTS path builds against an espeak-ng fork for phonemization. Upstream espeak-ng is GPL-3.0.
- Decision: do not hide the issue. Continue TTS experiments (research only) but record the risk; seek legal review before any commercial use. Evaluate TTS models that do not need espeak-ng (e.g. char-based VITS, other architectures).
- Consequences: Piper/Kokoro may be unusable commercially even though their weights are permissively licensed.

## D-005 — Model license exclusions
- Date: 2026-10-06 · Status: accepted
- Excluded for the commercial objective: `vits-mms-fra` (CC-BY-NC-4.0), Piper `fr_FR-tom` (AGPLv3). CC-BY-SA (Piper upmc) requires review. CC-BY models are acceptable with attribution.

## D-006 — Architecture: provider traits behind a Tauri command contract
- Date: 2026-10-06 · Status: accepted (details in `docs/M0_FEASIBILITY.md` §8)
- Decision: Rust traits `SpeechToTextProvider` / `TextToSpeechProvider`, adapters per engine, UI only sees an engine-agnostic JSON contract. Models downloaded on demand, never bundled.

## D-007 — Audio capture strategy
- Date: 2026-10-06 · Status: proposed (to verify in M4)
- Decision: try WebView `getUserMedia` first (least native code); fall back to `cpal` if microphone permission or quality is a problem.
