# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| whisper-cpp-tiny | beam search (5 beams) | 95 | 0 | 17.1 % | 17.0 % | 6.7 % | 0 | 17 | 34 | 0.213 | 1070 | 1260 | 107 | 290 | 3.7 |
| whisper-cpp-tiny | greedy (beam 1) | 95 | 0 | 20.0 % | 19.8 % | 8.0 % | 0 | 21 | 41 | 0.181 | 921 | 1104 | 106 | 256 | 3.7 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 95 | 0 | 8.3 % | 8.3 % | 2.5 % | 0 | 11 | 22 | 0.430 | 2078 | 2473 | 93 | 311 | 3.9 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 95 | 0 | 9.8 % | 9.4 % | 3.2 % | 0 | 12 | 24 | 0.386 | 1879 | 2232 | 93 | 260 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 95 | 0 | 3.3 % | 3.4 % | 1.3 % | 0 | 7 | 10 | 1.517 | 7261 | 8593 | 211 | 577 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 95 | 0 | 3.8 % | 3.9 % | 1.5 % | 0 | 8 | 12 | 1.405 | 6811 | 7898 | 213 | 454 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | en-general | en-technical | fr-administrative | fr-dictation | fr-general | fr-with-english-terms | fr-legal | fr-medical |
|---|---|---|---|---|---|---|---|---|---|---|
| whisper-cpp-tiny | beam search (5 beams) | 0.0 % (0) | 0.8 % (0) | 8.1 % (1) | 18.9 % (2) | 21.4 % (1) | 20.5 % (3) | 20.0 % (2) | 5.9 % (1) | 37.8 % (7) |
| whisper-cpp-tiny | greedy (beam 1) | 0.0 % (0) | 0.8 % (0) | 8.1 % (1) | 27.0 % (4) | 23.4 % (2) | 25.4 % (3) | 22.0 % (3) | 13.2 % (2) | 40.5 % (6) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 4.9 % (1) | 4.1 % (1) | 6.8 % (1) | 11.0 % (2) | 8.8 % (0) | 8.5 % (0) | 2.9 % (0) | 18.0 % (5) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 4.9 % (1) | 4.1 % (1) | 8.1 % (2) | 13.8 % (2) | 11.7 % (1) | 10.0 % (0) | 7.4 % (0) | 16.2 % (4) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 1.6 % (1) | 5.4 % (1) | 1.4 % (0) | 4.1 % (0) | 4.9 % (2) | 3.0 % (0) | 0.0 % (0) | 4.5 % (2) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 1.9 % (1) | 1.6 % (1) | 6.8 % (1) | 1.4 % (0) | 4.1 % (0) | 3.9 % (2) | 4.5 % (0) | 0.0 % (0) | 7.2 % (3) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `fr-it-03-owner` (median WER 50 %): reference "L'API REST renvoie une erreur 500 quand le token expire."; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "API REST renvoie une erreur 500 quand le token expire."
- `fr-med-03-owner` (median WER 46 %): reference "La tension artérielle est de 14 sur 9, il s'agit d'une hypertension artérielle."; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "La tension artérielle est de 14 sur 9. Il s'agit d'une hypertension artérielle."
- `fr-it-18-owner` (median WER 43 %): reference "Le service redémarre automatiquement en cas d'erreur."; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "Le service redémarre automatiquement en cas d'erreur."
- `fr-med-09-owner` (median WER 40 %): reference "L'ordonnance mentionne de l'ibuprofène 400 mg en cas de douleur."; best output (whisper-cpp-base-q5_1, beam search (5 beams)): "L'ordonnance mentionne de ibuprofne, 400 mg en 4 douleurs."
- `fr-gen-q-02-owner` (median WER 38 %): reference "Peux-tu ouvrir le dossier de madame Dupont ?"; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "Peux-tu ouvrir le dossier de madame Dupont ?"
- `fr-it-02-owner` (median WER 38 %): reference "Peux-tu redémarrer le cluster Kubernetes avant midi ?"; best output (whisper-cpp-tiny, beam search (5 beams)): "Pouture, démarré, le cluster Kubernetes, avant midi."
- `fr-gen-q-12-owner` (median WER 33 %): reference "Quelle est la météo prévue pour demain à Lausanne ?"; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "Quel est la météo prévue pour demain à Lausanne ?"
- `fr-gen-q-09-owner` (median WER 25 %): reference "Qui m'a appelé ce matin avant 10 heures ?"; best output (whisper-cpp-small-q5_1, beam search (5 beams)): "Qui m'a appelé ce matin avant 10h?"

## Critical flags to review (repetition 1, first 6 per configuration)

- whisper-cpp-tiny / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension art rielle`: expected term term not found; closest text is 94 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-gen-q-03-owner: Unit expected `h` found `(absent)`: a unit was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-gen-q-09-owner: Number expected `10` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-gen-q-09-owner: Unit expected `h` found `(absent)`: a unit was lost
- whisper-cpp-tiny / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-04-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-06-owner: Number expected `(none)` found `8`: a number was added
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension art rielle`: expected term term not found; closest text is 94 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-adm-02-owner: Number expected `2026` found `2016`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension art riel`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-adm-02-owner: Number expected `2026` found `2016`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `45`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension art鋒iel`: expected term term not found; closest text is 76 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-gen-s-08-owner: Unit expected `(none)` found `m`: a unit was added
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-med-04-owner: KeyTerm expected `paracétamol` found `paracetamol`: expected drug term not found; closest text is 91 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-gen-s-08-owner: Unit expected `(none)` found `m`: a unit was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-med-04-owner: KeyTerm expected `paracétamol` found `paracetamol`: expected drug term not found; closest text is 91 % similar (probable misspelling)
