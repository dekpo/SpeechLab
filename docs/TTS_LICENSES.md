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
| Piper fr_FR mls medium | fr | CC BY 4.0 (Multilingual LibriSpeech) | **trained from scratch** | not stated separately | espeak-ng | would be `attribution` | no: not in the sherpa-onnx release, needs a conversion step |
| Piper en_US kristin medium | en | public domain (LibriVox) | trained from scratch | not stated separately | espeak-ng | would be `clear` | no: available in the sherpa-onnx release (64 MB), not downloaded yet |
| Piper fr_FR upmc medium | fr | CC BY-SA 4.0 | fine-tuned from Lessac | not stated separately | espeak-ng | review (share-alike plus Lessac) | no |
| Piper en_US lessac medium | en | Blizzard 2013 **research** licence agreement | from scratch | not stated separately | espeak-ng | review at best (licence granted to a named person or organisation; clauses not read in full) | no |
| Piper en_US amy, en_GB alan | en | "See URL" (not read) | fine-tuned from Lessac | not stated | espeak-ng | review (unread) | no |
| Piper fr_FR tom | fr | AGPL-3.0 | see URL | AGPL | espeak-ng | excluded | no |
| MMS fra (`vits-mms-fra`) | fr | CC BY-NC 4.0 | n/a | non-commercial | n/a | excluded | no |
| Piper en_US ryan, en_US hfc_female | en | CC BY-NC-SA 4.0 | ryan from Lessac; hfc from Lessac | non-commercial | espeak-ng | excluded | no |
| Piper fr_FR miro, tjiho | fr | no readable card | unknown | unknown | espeak-ng | review (unreadable) | no |
| Kitten TTS nano (en) | en | not read | not read | Apache-2.0 (model card) | espeak-ng (to confirm) | not rated: English only | no |
| Supertonic | multi | not read | not read | OpenRAIL-M (use-based restrictions) | none known | review | no |
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
5. The Piper `fr_FR-mls-medium` voice is **not in the sherpa-onnx release** (checked against the release
   asset list). Using it would need its checkpoint converted for sherpa-onnx (a Python tool and package
   installation, owner's approval needed first). It is the cleanest French Piper voice found (CC BY 4.0,
   trained from scratch): worth doing if the Coqui voice disappoints. The English `kristin` voice (public
   domain, from scratch) IS in the release and can be added with a plain download.

## Sources read (2026-10-08)

Piper voice cards `MODEL_CARD` in the `rhasspy/piper-voices` repository on Hugging Face (fr_FR siwis
medium and low, gilles, upmc, tom, mls; en_US libritts_r, lessac, ljspeech, ryan, hfc_female, kristin);
the Blizzard 2013 Lessac licence page (first part); Kokoro-82M `README.md` and `VOICES.md`; the Kokoro
v1.1-zh card; the CSS10 repository README and licence file; the Coqui model list (`.models.json`) and
the NeonGecko French VITS model card; the `config.json` shipped in the `vits-coqui-fr-css10` package
(read after download); Kitten, Supertonic and Pocket model cards (partly unreadable).
