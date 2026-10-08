# TTS listening notes (owner's opinion, recorded verbatim)

**These are opinions, not measurements.** One listener (the owner), 8 voices heard blind under codes
A to H (page `benchmark/tts-samples/index.html`, built by `scripts/tts_listening.py`), 4 clips per voice
(sentence with a drug name and numbers, date and time with another drug name, three drug names, and the
first sentence at speed 1.5), 2026-10-08. The codes were resolved only in the pasted result. The voices'
commercial-use ratings are in `docs/TTS_LICENSES.md`. Scale 1 (very bad) to 5 (very good).

## Scores

| Code | Voice | Lang | Licence rating | Clarity | Natural | Accent / pronunciation | Drug names | Speed 1.5 pleasant |
|---|---|---|---|---|---|---|---|---|
| A | Kokoro `ff_siwis` | fr | review | 4 | 3 | 4 | well pronounced | 4 |
| B | Piper siwis medium | fr | review | **5** | **5** | **5** | well pronounced | **5** |
| C | Coqui css10 | fr | attribution | 3 | 4 | **2** | approximate but recognisable | 4 |
| D | Piper gilles low | fr | excluded | 4 | **5** | 3 | well pronounced | **5** |
| E | Kokoro `af_heart` | en | review | 4 | 4 | 4 | well pronounced | 4 |
| F | Kokoro `am_adam` | en | review | 4 | 4 | 4 | well pronounced | 4 |
| G | Piper ljspeech medium | en | clear | **5** | 4 | **5** | well pronounced | **5** |
| H | Piper libritts_r (speaker 0) | en | attribution | **5** | **5** | **5** | well pronounced | **5** |

Owner's choice: **B, D, G and H** ("Je choisirais B D G et H").

## Comments (original French, with an English gloss)

- **A, Kokoro fr**: "Voix correcte mais présence d'un sifflement ou grésillement désagréable pendant la lecture" (acceptable voice, but an unpleasant hiss or crackle during reading).
- **B, Piper siwis**: "Voix très agréable et propre pas de sifflement pendant la lecture très bonne intonation" (very pleasant and clean, no hiss, very good intonation).
- **C, Coqui css10**: "L'accent est très marqué et le mot 'Rendez-vous' est mal prononcé, les chiffres 12, 14 et 30 ne sont pas prononcés" (strong accent, "Rendez-vous" mispronounced, the numbers 12, 14 and 30 are not spoken).
- **D, Piper gilles**: "L'accent est particulier la voix roule les 'R' mais globalement c'est très correcte" (peculiar accent, rolls the R, but overall very correct).
- **E and F, Kokoro en**: "C'est globalement correcte mais il y a un léger sifflement grésillement dans la lecture audio" (generally correct, but a slight hiss or crackle in the audio).
- **G, Piper ljspeech**: "C'est très bien bonne qualité sonore et bonne intonation" (very good, good sound quality and intonation).
- **H, Piper libritts_r**: "C'est très bien bonne qualité sonore et bonne intonation encore plus naturelle que la voix précédente" (very good, good sound quality and intonation, even more natural than the previous voice).

## What the notes show (analysis, not opinion)

1. **English: no conflict between quality and licence.** The two voices preferred (G `clear`, H `attribution`) are the cleanest ones, and both beat the Kokoro voices (`review`) on every criterion except one tie.
2. **French: the conflict is real.** The two French voices the owner preferred (B `review`, D `excluded`) are not usable under the licence-first rule (D-039, D-041). The only French voice rated freely usable (C, Coqui css10, `attribution`) is the one with the lowest scores (clarity 3, accent 2), mispronounces a common word and drops digits (already measured: I-055).
3. **Kokoro has an audible artefact**: a hiss or crackle was heard on all three Kokoro voices (A, E, F) and on no Piper or Coqui voice. Cause not isolated; a candidate cause is the int8 quantisation, NOT verified (the fp32 package, about 330 MB, was not tried): I-057. Together with its licence rating and its slowness (RTF 1.1 to 2.1), Kokoro is deprioritised.
4. **Speed 1.5 is acceptable for every voice** (4 or 5 everywhere); the speed control is usable.
5. **Drug names**: the owner heard them well pronounced on seven voices out of eight, approximate on Coqui. The speech-recogniser round trip (`roundtrip-parakeet.txt`) had counted many more failures: the machine check was too pessimistic to be a proxy for a human ear (the recogniser itself misspells drug names), so it is kept only as a rough sanity check.
6. Limits: one listener, three sentences of text, no listening to long texts, no reading of a real report, no comparison of speaker numbers other than libritts_r speaker 0.

## Session 2 (2026-10-08): `kristin` against the other English voices, Coqui normaliser ON against OFF, `mls` speakers — PENDING

Page: `benchmark/tts-samples/session2/index.html` (built by `python -I -X utf8 scripts/tts_listening.py --session 2`; audio is git-ignored). Eight voices under codes A to H, three clips each, blind. The owner's notes are to be pasted here VERBATIM when received, as opinion, in the same format as session 1 (scores, choice, comments, original French with an English gloss).

What is compared (the codes are resolved only in the pasted result):
- English, to choose the best public-domain or attributable voice: Piper `kristin` (`clear`), Piper `ljspeech` (`clear`), Piper `libritts_r` speaker 0 (`attribution`).
- French, Coqui css10 with the text normaliser ON and OFF (same voice, digit-heavy sentences: a date and a time and a dose, then an invoice number with a price and a phone number).
- French, three speakers (24, 65, 117) of the converted Piper `mls` voice (CC BY 4.0, `attribution`), picked as the best of 125 by a recogniser proxy (33 % word error rate each, against 6.7 % for Coqui on the same sentences). The proxy says nothing about naturalness: this is what the owner's ear decides.

Machine facts that go with it (measurements, not opinion): `kristin`, `ljspeech`, `libritts_r` and Coqui were measured together at 17 to 25 times faster than real time on this CPU (`benchmark/results/20261008-141002-tts-m6d-overhead/overhead.md`); the `mls` speakers ran at RTF 0.05 to 0.09 in the speaker scan (single runs, not a controlled measurement); with the normaliser on, a recogniser heard the key numbers of six sentences in 28/36 and 30/36 cases, against 0/36 with it off (`normaliser-roundtrip-parakeet.txt`).

Decisions the notes should allow: the French voice to offer in the beta (Coqui with the normaliser, a converted `mls` speaker, or none until a better free voice exists); which English voice to make the default.
