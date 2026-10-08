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

### Owner's decision of 2026-10-08 (D-043, amends D-041 for two voices)

The owner keeps **Piper siwis (`review`) and gilles (`excluded`)** as the French voices for now and will **re-train them from the clean base checkpoint before any commercial release**. Until then they are for development, evaluation and non-commercial use; the ratings above do not change and the app keeps warning. A commercial release is blocked until they are replaced (re-trained from `_base_model`, or cleared in writing by the rights holders, or after a legal reading) and re-listened blind. For every other voice the D-041 stance is unchanged.

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
5. The Piper `fr_FR-mls-medium` voice is **not in the sherpa-onnx release** (the release has 644 assets; its French
   voices are listed in "French voice: lineage problem and ways out" below, none is both clean and `mls`). It was converted on this machine on
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

## French voice: lineage problem and ways out (read 2026-10-08, after the owner's session 2)

**The situation, with the evidence.** Of the French voices installed, the two the owner judges good are the two whose PARENT checkpoint is restricted; the data of both is permissive.

| Voice | Data licence | Parent checkpoint (card) | Parent's licence | Rating |
|---|---|---|---|---|
| Piper siwis medium (female) | SIWIS French Speech Synthesis Database, CC BY 4.0 (VERIFIED: the dataset README says "usable for any purpose, under CC BY 4.0"; 9,750 utterances, more than 10 h, one professional speaker) | "Finetuned from U.S. English lessac voice (medium quality)" | Lessac data = Blizzard 2013 Research Licence Agreement (see below) | review |
| Piper gilles low (male) | Kaggle "French single speaker speech dataset", CC0 (card); appears to be the CSS10 French corpus built from LibriVox (public domain), only the dataset title was checked | "Finetuned from U.S. English Ryan voice (low quality)" | Ryan data CC BY-NC-SA 4.0 | excluded |
| Coqui css10 | CSS10 French (public domain recordings) | not stated | BSD-3-Clause (model) | attribution (judged 3/3/3/3 with the normaliser) |
| Piper mls (converted) | CC BY 4.0 | trained from scratch | none | attribution (judged 1 to 2: rejected) |

**What the Blizzard licence says** (page read 2026-10-08 through a page summary, quotes as returned): a "Research Licence Agreement" between Voice Factory International, Lessac Technologies and the user; "Research Purposes means only those purposes associated with research and exploration ... using, incorporating or based upon the Materials"; it "excludes loading, executing, storing, transmitting, displaying, copying, reverse engineering, developing, adapting, amending or otherwise using the Materials for any commercial purpose, including the development, marketing, commercialisation, sale or licencing of voice synthesis or speech recognition products or services"; the licensee is a named user who gives a postal address; redistribution of the Materials needs the licensors' written consent; no clause on trained models was found. This is a technical reading, not legal advice: the licence speaks of the Materials (the recordings) and of works "based upon" them, and says nothing explicit about weights.

**What the Piper maintainer says** (read: rhasspy/piper discussion 271 and rhasspy/piper-voices discussion 94): "I'm not a lawyer, so I can't offer any legal advice", "I have no idea how licenses are supposed to apply in the context of machine learning", "This is something that I don't think has been tested yet legally", and: "Either way, I've trained a new base model from LibriTTS-R (CC-BY) and will slowly be re-training many of the Piper voices so that they can at least be used for commercial purposes." The Piper documentation itself says Piper is "intended for personal use and text to speech research only" and that each voice's `MODEL_CARD` must be read. The card of the French `siwis` voice in `piper-voices` still says "Finetuned from ... lessac" (read 2026-10-08): it was not re-trained from the clean base at that date.

**A clean parent checkpoint exists and is public**: `rhasspy/piper-checkpoints`, folder `_base_model` (repository licence MIT), `base_model.ckpt` (933 MB) with `config.json`; its card: English (US), 904 speakers, medium, 22,050 Hz, dataset OpenSLR 141 (LibriTTS-R) CC BY 4.0, "Trained from scratch on LibriTTS-R train-clean-360 with MRD and SDP, uses vowel clusters, so only compatible with Piper 1.5.0+". The same repository holds the French `mls` checkpoint (925 MB, trained from scratch, CC BY 4.0), the `siwis` checkpoint (training command included) and the `libritts_r` voice checkpoint. The installed `libritts_r` voice is trained from this base and works in sherpa-onnx here, which shows sherpa-onnx reads voices of this family.

**Corrections of earlier statements (my errors).** (1) The M6d log said the sherpa-onnx `tts-models` release lists only `vits-coqui-fr-css10` and `vits-mms-fra` for French: that came from a page summary that covered only the first 10 % of the page. The release has 644 assets. Its French voices are, besides those already examined: `fr_FR-tom-medium` (AGPL-3.0), `fr_FR-upmc-medium` (CC BY-SA 4.0, fine-tuned from Lessac), `fr_FR-siwis-low` (as siwis), `fr_FR-tjiho-model1/2/3` (the `LICENSE.txt` shipped with the voice is the GNU AGPL-3.0, converted from the GitHub project tjiho/French-tts-model-piper), `fr_FR-miro-high` (voice of the OpenVoiceOS project, trained on a SYNTHETIC dataset; Hugging Face metadata `cc-by-nc-nd-4.0`, while the sherpa-onnx packaging script writes "CC BY-NC-SA 4.0, commercial use not allowed"). None of them is usable under D-041, so the conclusion stands, now with the complete list (`gh api` on the release, VERIFIED). (2) `tjiho` and `miro` are no longer "unreadable": AGPL-3.0 and non-commercial, both `excluded`.

**Ways out, in my order of preference** (none decided; D-043 is a proposal):
1. **Re-train the two voices the owner likes from the clean base.** Same speakers and data (SIWIS CC BY 4.0; the CSS10-French-based data of gilles, CC0), parent = `_base_model` (CC BY 4.0 data, from scratch) instead of Lessac or Ryan: the lineage becomes CC BY / CC0 only, rating `attribution`. Needs a GPU (Piper's documentation reports 8 GB of video memory as the minimum that worked; this laptop has none), the Piper 1.5 training code (`piper1-gpl`, GPL-3.0: a training tool, not shipped), 2.67 GB of SIWIS audio and the 933 MB checkpoint, the ONNX export and our `scripts/convert_piper_voice.py`. Time and cost are NOT VERIFIED: the siwis voice in `piper-checkpoints` was trained for 3,304 epochs on a 24 to 48 GB GPU; a fine-tune needs far fewer, but how many for a good result is unknown. Quality is NOT VERIFIED: a different parent may sound different from the voices the owner liked, so a new blind listening is needed. A French parent (the `mls` checkpoint) is an alternative base worth one comparison run.
2. **Ask the rights holders** for a written statement (Lessac Technologies for the checkpoint; the Piper maintainer, who already announced retraining). Free, slow, outcome unknown.
3. **Accept the risk** for siwis only (gilles stays excluded): the owner's stance D-041 refuses it for the beta.
4. **Beta with what is clean**: English voices (libritts_r, kristin, ljspeech) and, for French, Coqui css10 with the normaliser (judged 3 out of 5) until a re-trained voice exists.

## Paid French voices (prices read 2026-10-08; NOT checked on the vendors' own price pages)

Prices come from web search results, mostly third-party comparison sites, so they must be confirmed on each vendor's page before any decision. All figures are per million characters unless stated. Reference: about 3,000 characters per medical letter.

| Offer | Where it runs | Price read | Notes |
|---|---|---|---|
| Google Cloud Text-to-Speech | cloud | Standard and WaveNet $4; Neural2 $16; Chirp 3 HD $30; Studio $160; free tier 4M (Standard, WaveNet) and 1M (others) per month | French exists; text leaves the machine |
| Amazon Polly | cloud | Standard $4; Neural $16; Generative $30; Long-form $100 | French availability per engine not checked |
| Microsoft Azure AI Speech | cloud (also containers) | Neural $16; Neural HD $22 (was $30 until March 2026); custom neural $24 to $48; free tier 500K per month; commitment tiers down to about $7.50 | disconnected containers exist (offline-like), their terms and price were not found |
| OpenAI text to speech | cloud | tts-1 $15; tts-1-hd $30; gpt-4o-mini-tts about $15 (token-billed) | |
| ElevenLabs | cloud | API $0.08 per 1,000 characters (Eleven v3 and Multilingual v2), $0.04 for Flash/Turbo; commercial licence from the Starter plan at $6 per month (30,000 credits); Creator $22, Pro $99, Scale $299, Business $990 per month | best-known naturalness; no offline mode |
| Acapela Group (French voices such as Manon, Louis) | on device, offline SDK | quote only: SDK paid once with first-year maintenance, plus royalties for embedded apps (percentage of the app price depending on units, voices, languages) | the nearest paid equivalent of this project's constraint (offline) |
| ReadSpeaker speechEngine SDK, CereProc, Cerence | on device, offline SDK | quote only (CereProc consumer voices from about GBP 10 per month, not a redistribution licence) | |

Order of magnitude, 1,000 letters of 3,000 characters per month (3M characters): Google WaveNet about $12, Azure Neural about $48, OpenAI about $45, Google Chirp 3 HD or Amazon Generative about $90, ElevenLabs API about $240 (v3) or $120 (Flash).

Constraints of this project that these offers meet or not: every cloud offer sends the text (here medical text) to a third party and needs the network, which breaks the rule "everything runs locally, offline, at inference" (`Plan.md`, AGENTS.md section 4) and raises health-data questions; only the on-device SDKs (Acapela, ReadSpeaker, CereProc, Cerence, Azure disconnected containers) respect the offline rule, and none publishes a price.

## Sources read (2026-10-08)

Piper voice cards `MODEL_CARD` in the `rhasspy/piper-voices` repository on Hugging Face (fr_FR siwis
medium and low, gilles, upmc, tom, mls; en_US libritts_r, lessac, ljspeech, ryan, hfc_female, kristin), the `mls` card and JSON re-read 2026-10-08;
the Blizzard 2013 licence page (second reading), the Piper discussions rhasspy/piper 271 and rhasspy/piper-voices 94, the `piper-checkpoints` repository (`_base_model`, siwis, mls, libritts_r cards), the SIWIS dataset page and README (Edinburgh DataShare), the tjiho `LICENSE.txt` and the sherpa-onnx `scripts/piper/generate.py`, the release asset list (`gh api`), the Chatterbox, Chatterbox-TTS-French and Qwen3-TTS-12Hz-0.6B-Base model cards, the Emilia dataset card, the `chatterbox.cpp` README and the sherpa-onnx `tts-models` release list (2026-10-08);
the Blizzard 2013 Lessac licence page (first part); Kokoro-82M `README.md` and `VOICES.md`; the Kokoro
v1.1-zh card; the CSS10 repository README and licence file; the Coqui model list (`.models.json`) and
the NeonGecko French VITS model card; the `config.json` shipped in the `vits-coqui-fr-css10` package
(read after download); Kitten, Supertonic and Pocket model cards (partly unreadable).
