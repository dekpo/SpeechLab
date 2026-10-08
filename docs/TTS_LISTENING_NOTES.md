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

## Session 2 (2026-10-08): `kristin` against the other English voices, Coqui normaliser ON against OFF, `mls` speakers — RESULTS (owner's opinion, recorded verbatim)

Page: `benchmark/tts-samples/session2/index.html` (built by `python -I -X utf8 scripts/tts_listening.py --session 2`; audio is git-ignored). Eight voices under codes A to H, three clips each, blind, one listener. Scale 1 (very bad) to 5 (very good). The owner's words about the page: "Les voix proposées dans le fichier benchmark/tts-samples/session2/index.html sont médiocres" (the voices offered in the session 2 file are mediocre). No free-text comment was written for any voice.

| Code | Voice | Lang | Licence rating | Clarity | Natural | Accent | Numbers, date, time, price, phone | Pleasant for long listening |
|---|---|---|---|---|---|---|---|---|
| A | Piper `mls` speaker 24 | fr | attribution | 2 | **1** | 3 | approximate | 2 |
| B | Piper `mls` speaker 117 | fr | attribution | 2 | **1** | 2 | approximate | 2 |
| C | Coqui css10, normaliser ON | fr | attribution | 3 | 3 | 3 | **well said** | 3 |
| D | Piper `mls` speaker 65 | fr | attribution | 2 | **1** | 3 | wrong, absent or unintelligible | **1** |
| E | Coqui css10, normaliser OFF | fr | attribution | **1** | 2 | 2 | wrong, absent or unintelligible | **1** |
| F | Piper `ljspeech` | en | clear | **4** | **4** | **4** | well said | 3 |
| G | Piper `libritts_r` speaker 0 | en | attribution | 3 | 3 | 3 | well said | 3 |
| H | Piper `kristin` | en | clear | 3 | 2 | 3 | well said | 3 |

General remark field: empty.

### Owner's note on the interface tests (verbatim, original French)

"Dans les tests de l'interface la meilleure voix française est Piper fr_FR siwis (medium) pour la femme et Piper fr_FR gilles (low) pour l'homme à la vitesse par défaut et la réécritures des nombres des dates des heures et des unités en toutes lettres. Et en anglais Piper en_US libritts_r (medium) vitesse 0.9 avec différents locuteurs (hommes/femmes) et la réécritures des nombres des dates des heures et des unités en toutes lettres. Piper en_US kristin (medium) et iper en_US ljspeech (medium) toujours avec la réécritures des nombres des dates des heures et des unités en toutes lettres sont correctes donc nous avons un problème pour la licence en français c'est exacte ? Car malheureusement toutes les autres voix ne sont pas satisfaisantes."

English gloss: in the interface tests the best French voices are Piper siwis (female) and Piper gilles (male) at default speed with numbers, dates, times and units rewritten in words; in English Piper libritts_r at speed 0.9 with several speakers (men and women), and kristin and ljspeech, always with the rewriting on, are correct; therefore there is a licence problem for French. All the other voices are unsatisfactory.

### What the notes show (analysis, not opinion)

1. **The normaliser works as intended on the voice that needs it**: Coqui with it ON (C) is rated "numbers well said" and 3/3/3/3, with it OFF (E) clarity 1 and numbers "wrong, absent or unintelligible". It does not make Coqui good (3 out of 5 at best).
2. **The converted `mls` voice is rejected**: every speaker is 1 or 2 on naturalness and 2 on clarity, as the recogniser proxy (best of 125 at 33 % word error rate against 6.7 % for Coqui) suggested. `mls` is not a way out (I-060).
3. **English**: `ljspeech` (F, `clear`) is the best of this session (4, 4, 4); `libritts_r` speaker 0 and `kristin` score 3. The owner's interface note, which listens to more than three clips and uses other speakers and speed 0.9, ranks `libritts_r` first and `kristin` and `ljspeech` correct: English has three usable voices, one `attribution` and two `clear`.
4. **The owner also uses the rewriting of numbers, dates, times and units on the phonemizer voices** (Piper siwis, gilles, libritts_r, kristin, ljspeech), not only on Coqui: the manifest default (off for phonemizer voices) does not match this use (proposal in D-043).
5. **French: the licence conflict is confirmed by the owner's own reading**: the only voices judged good in French are siwis (`review`) and gilles (`excluded`); every French voice that could be offered under D-041 is judged unsatisfactory. Resolution options and the evidence behind them: `docs/TTS_LICENSES.md` ("French voice: lineage problem and ways out"), D-043 (proposed) and I-063.
6. Limits: one listener, three clips per voice in this session; the interface test used the owner's own texts.
