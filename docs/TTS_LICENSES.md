# Text-to-speech voices: commercial-use licence review

Owner's rule (2026-10-08): the TTS benchmark is oriented toward voices that are free or that carry the
fewest constraints for commercial use. Every voice therefore gets an explicit rating, with the evidence
behind it. The rating is shown in the app (voice table and voice list), stored in
`src-tauri/models-manifest.json` (`licenseTier`, `licenseNotes`, `phonemizer`), checked by a unit test
(a voice without a rating, a reason and a stated phonemizer cannot be added) and printed next to every
measurement. **This is a technical reading of public licence texts and model cards, not legal advice**:
anything rated `review` needs a lawyer before a product ships it.

## Ratings

| Rating | Meaning |
|---|---|
| `clear` | Training data public domain or CC0, voice trained from scratch, no attribution duty. |
| `attribution` | Commercial use allowed, but a credit or licence notice must travel with the product (CC-BY, BSD-3). |
| `review` | Commercial use plausible, but one point needs a legal reading: lineage of the weights, share-alike, provenance of the training audio, or a licence not read. |
| `excluded` | Non-commercial or copyleft data in the voice or its lineage, or a licence the owner has not accepted. May stay installed for comparison only; never recommended. |

The **phonemizer** is rated separately because it is a different obligation: Piper and Kokoro voices
turn text into phonemes with espeak-ng (GPL-3.0). A character-based voice does not use it.

## How a voice is checked (rule for every new candidate)

1. Read the voice card: dataset, dataset licence, **"Finetuned from ..." / "Trained from scratch"**.
   A permissive data licence does not clear weights that were initialised from a restricted voice.
2. Follow the lineage one step: read the licence of the parent voice's data.
3. Read the model card of the runtime family (provenance of training audio, extra data used).
4. Record the phonemizer the package needs at synthesis time.
5. Write the rating and the reason in the manifest, then add the row below.

## Voices examined (read 2026-10-08)

| Voice | Lang | Data licence | Lineage (from the card) | Weights licence | Phonemizer | Rating | Installed |
|---|---|---|---|---|---|---|---|
| Coqui VITS fr css10 (`vits-coqui-fr-css10`) | fr | CSS10 French = LibriVox recordings (public domain); the CSS10 repository is Apache-2.0 | not stated by the card | BSD-3-Clause (model card, Coqui model list) | **none** (character-based, `use_phonemes: false`) | **attribution** | yes |
| Piper en_US ljspeech medium | en | public domain (LJ Speech) | trained from scratch | not stated separately (voice cards give the dataset licence only) | espeak-ng | **clear** | yes |
| Piper en_US libritts_r medium | en | CC BY 4.0 (LibriTTS-R) | no fine-tuning line on the card | not stated separately | espeak-ng | **attribution** | yes |
| Piper fr_FR siwis medium | fr | CC BY 4.0 (SIWIS) | **fine-tuned from the English Lessac voice** | not stated separately | espeak-ng | **review** | yes |
| Piper fr_FR gilles low | fr | CC0 (same corpus as CSS10 French) | **fine-tuned from the English Ryan voice** | not stated separately | espeak-ng | **excluded** | yes (comparison only) |
| Kokoro multi-lang v1.0 (int8) | fr, en | Apache-2.0 / MIT audio, **synthetic audio from closed TTS services**, SIWIS (CC BY 4.0), Koniwa (CC BY 3.0) | not applicable | Apache-2.0 | espeak-ng | **review** | yes |
| Piper fr_FR mls medium | fr | CC BY 4.0 (Multilingual LibriSpeech, OpenSLR 94; 125 speakers) | **trained from scratch** (card read again 2026-10-08) | not stated separately | espeak-ng | **attribution** | yes, converted locally 2026-10-08 with `scripts/convert_piper_voice.py` (owner's approval); weak by a recogniser proxy, I-060 |
| Piper en_US kristin medium | en | public domain (LibriVox) | trained from scratch | not stated separately | espeak-ng | **clear** | yes (installed 2026-10-08 on the owner's approval; measured 2026-10-08, RTF 0.055 to 0.064, listening session 2 pending) |
| Piper fr_FR upmc medium | fr | CC BY-SA 4.0 | fine-tuned from Lessac | not stated separately | espeak-ng | review (share-alike plus Lessac) | no |
| Piper en_US lessac medium | en | Blizzard 2013 **research** licence agreement | from scratch | not stated separately | espeak-ng | review at best (licence granted to a named person or organisation; clauses not read in full) | no |
| Piper en_US amy, en_GB alan | en | "See URL" (not read) | fine-tuned from Lessac | not stated | espeak-ng | review (unread) | no |
| Piper fr_FR tom | fr | AGPL-3.0 | see URL | AGPL | espeak-ng | excluded | no |
| MMS fra (`vits-mms-fra`) | fr | CC BY-NC 4.0 | n/a | non-commercial | n/a | excluded | no |
| Piper en_US ryan, en_US hfc_female | en | CC BY-NC-SA 4.0 | ryan from Lessac; hfc from Lessac | non-commercial | espeak-ng | excluded | no |
| Piper fr_FR miro, tjiho | fr | no readable card | unknown | unknown | espeak-ng | review (unreadable) | no |
| Kitten TTS nano (en) | en | not read | not read | Apache-2.0 (model card) | espeak-ng (to confirm) | not rated: English only | no |
| Supertonic | multi | not read | not read | OpenRAIL-M (use-based restrictions) | none known | review | no |
| Chatterbox (Resemble AI, base English-only and multilingual with French) | multi incl. fr | card: "0.5M hours of cleaned data" from "freely available data on the internet", no data licence stated | not stated | MIT (card) | none read (not a sherpa-onnx model; PyTorch, ONNX, GGML port `chatterbox.cpp` under MIT); **every output carries the Perth neural watermark**; voice cloning from a reference clip (only restriction stated: "don't use this model to do bad things") | review (data licence unstated); 0.5B, files about 2.1 GB, the port reports RTF 4.32 on an Apple M4 with 4 threads: slower than real time, not measured here | no |
| Chatterbox-TTS-French (community fine-tune, `Thomcles`) | fr | card: French part of the Emilia dataset, 1,400 h, "CC BY 4.0"; but Emilia is CC BY-NC-4.0 (only the YODAS subset is CC BY 4.0), audio from videos and podcasts whose copyright stays with their owners; the card does not say which subset | fine-tune of Chatterbox | CC BY 4.0 (card, data) and MIT (base) | as above | review, likely excluded if the non-commercial Emilia was used (NOT VERIFIED which subset) | no |
| Qwen3-TTS (0.6B and 1.7B) | 10 languages including French (card) | card: "over 5 million hours of speech data", no data licence stated | not stated | Apache-2.0 (card) | none read (not a sherpa-onnx model); voice cloning from a reference clip, no watermark mentioned | review (data licence unstated); 0.9B parameters in BF16 for the 0.6B model, card shows CUDA and flash-attention, community GGUF and ONNX ports exist (`qwentts.cpp`, LunaVox), no CPU real-time factor found (only RTF 0.35 on an RTX 5050 GPU) | no |
| Pocket TTS | multi | not read | not read | CC-BY-4.0, gated (M0 note, card not readable now) | not read | not rated | no |

Runtime components for context: sherpa-onnx Apache-2.0, ONNX Runtime MIT, espeak-ng GPL-3.0 (inside
the standard sherpa-onnx libraries, D-012).

## What this review changed (corrections of earlier statements)

- Earlier entries (D-038, the M6 log) described Piper `siwis` as CC-BY and `gilles` as CC0 and offered
  them as the low-constraint choice. That read only the "Dataset" line of the cards. The cards also say
  both voices were **fine-tuned from other voices** (Lessac, Ryan) whose data is a research-only
  agreement and CC BY-NC-SA. `siwis` is now `review`, `gilles` is `excluded`.
- The best-documented clean choices found so far: English **Piper ljspeech** (`clear`) and French
  **Coqui css10** (`attribution`, and the only French voice that needs no phonemizer).

## Owner's stance (2026-10-08, D-041)

A beta is acceptable only if it needs no paid licence; no legal consultation for now; a budget may be considered later depending on users and revenue. Public domain is preferred. So only `clear` and `attribution` voices are offered by default; the voices the owner liked best in French (Piper siwis `review`, gilles `excluded`) stay out of any distribution (`docs/TTS_LISTENING_NOTES.md`).

## Open points (the owner or a lawyer must settle them)

1. **Is the GPL phonemizer still a problem for a character-based voice?** The Coqui voice never calls
   espeak-ng, but the standard sherpa-onnx static libraries still contain its code (the TTS-capable
   libraries are built with it). Whether shipping a binary that contains but never calls GPL code
   triggers the licence is a legal question. A technical way out (a sherpa-onnx build without the
   phonemizer, or another runtime for character-based VITS) was NOT tried (I-051, I-054).
2. **Lessac and Ryan lineage**: whether weights fine-tuned from a research-licence checkpoint carry the
   restriction (I-052).
3. **Kokoro training audio generated by closed TTS services**: their terms were not read (I-053).
4. **BSD-3 / CC-BY notices**: where a product shows credits (CC BY 4.0 SIWIS, LibriTTS-R, BSD-3 notice).
5. The Piper `fr_FR-mls-medium` voice is **not in the sherpa-onnx release** (the release lists only
   `vits-coqui-fr-css10` and the non-commercial `vits-mms-fra` for French). It was converted on this machine on
   the owner's approval (2026-10-08): isolated environment `vendor/py-convert` with the `onnx` package only,
   source files downloaded from the Piper repository (SHA-256 of the source `.onnx`
   `0ed223f78466917f2bae05ee90096ce69ab1fdeb251f55590d0e7422d234e162`), `scripts/convert_piper_voice.py convert`
   (procedure of the sherpa-onnx guide, first verified with `check` on the official kristin and ljspeech
   packages: identical tokens and metadata). To redo it: create the environment (`python -m venv vendor/py-convert`,
   `vendor/py-convert/Scripts/python.exe -m pip install onnx`), download `fr_FR-mls-medium.onnx`, `.onnx.json`
   and `MODEL_CARD` from `huggingface.co/rhasspy/piper-voices/tree/main/fr/fr_FR/mls/medium` into
   `vendor/piper-mls-source/`, then run the `convert` command with `--espeak-data` pointing at the
   `espeak-ng-data` folder of any installed Piper package and `--out-dir` at
   `<models>/vits-piper-fr_FR-mls-medium`. The voice is weak by a recogniser proxy (I-060).
6. **Chatterbox and Qwen3-TTS** cannot be offered as free voices on what was read: neither card states the
   licence of its training data, Chatterbox watermarks every output and both clone voices from a reference
   clip (a consent policy would be needed), the French Chatterbox fine-tune was trained on Emilia (CC BY-NC-4.0
   unless the YODAS subset was used, not stated), and neither runs faster than real time on a laptop CPU as far as
   was found (Chatterbox multilingual: reported RTF 4.32 on an Apple M4; Qwen3-TTS: no CPU figure). Not measured
   here, NOT VERIFIED.

## Sources read (2026-10-08)

Piper voice cards `MODEL_CARD` in the `rhasspy/piper-voices` repository on Hugging Face (fr_FR siwis
medium and low, gilles, upmc, tom, mls; en_US libritts_r, lessac, ljspeech, ryan, hfc_female, kristin), the `mls` card and JSON re-read 2026-10-08;
the Chatterbox, Chatterbox-TTS-French and Qwen3-TTS-12Hz-0.6B-Base model cards, the Emilia dataset card, the `chatterbox.cpp` README and the sherpa-onnx `tts-models` release list (2026-10-08);
the Blizzard 2013 Lessac licence page (first part); Kokoro-82M `README.md` and `VOICES.md`; the Kokoro
v1.1-zh card; the CSS10 repository README and licence file; the Coqui model list (`.models.json`) and
the NeonGecko French VITS model card; the `config.json` shipped in the `vits-coqui-fr-css10` package
(read after download); Kitten, Supertonic and Pocket model cards (partly unreadable).
