# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 95 | 0 | 25.4 % | 27.3 % | 16.2 % | 5 | 19 | 56 | 0.113 | 566 | 863 | 523 | 894 | 7.4 |
| sherpa-canary-180m-flash-int8 | greedy search | 95 | 0 | 5.9 % | 8.5 % | 2.5 % | 1 | 3 | 19 | 0.134 | 659 | 1507 | 1238 | 1088 | 7.4 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 95 | 0 | 2.3 % | 2.7 % | 0.8 % | 0 | 4 | 7 | 0.130 | 630 | 940 | 2463 | 1875 | 9.0 |
| whisper-cpp-tiny | beam search (5 beams) | 95 | 0 | 16.1 % | 16.7 % | 6.2 % | 0 | 13 | 40 | 0.177 | 857 | 1087 | 110 | 244 | 3.7 |
| whisper-cpp-tiny | greedy (beam 1) | 95 | 0 | 19.8 % | 19.5 % | 8.4 % | 1 | 18 | 53 | 0.149 | 705 | 918 | 112 | 214 | 3.5 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 95 | 0 | 8.4 % | 9.1 % | 3.2 % | 0 | 12 | 29 | 0.373 | 1807 | 2380 | 95 | 265 | 3.8 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 95 | 0 | 11.4 % | 11.9 % | 4.3 % | 0 | 15 | 38 | 0.355 | 1688 | 2319 | 97 | 217 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 95 | 0 | 3.9 % | 4.0 % | 1.5 % | 0 | 7 | 15 | 1.851 | 8283 | 14556 | 267 | 532 | 3.8 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 95 | 0 | 4.5 % | 4.7 % | 1.9 % | 0 | 7 | 17 | 1.563 | 7358 | 13245 | 269 | 411 | 3.8 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | en-general | en-technical | fr-administrative | fr-dictation | fr-general | fr-with-english-terms | fr-legal | fr-medical |
|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 1.9 % (0) | 36.6 % (2) | 28.4 % (1) | 24.3 % (3) | 26.9 % (2) | 19.5 % (3) | 26.5 % (2) | 23.5 % (1) | 31.5 % (5) |
| sherpa-canary-180m-flash-int8 | greedy search | 3.7 % (0) | 1.6 % (0) | 45.9 % (1) | 2.7 % (0) | 0.7 % (0) | 1.5 % (0) | 7.5 % (0) | 0.0 % (0) | 2.7 % (2) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 0.0 % (0) | 2.4 % (0) | 5.4 % (1) | 2.7 % (0) | 0.7 % (1) | 2.9 % (0) | 2.5 % (0) | 1.5 % (0) | 1.8 % (2) |
| whisper-cpp-tiny | beam search (5 beams) | 0.0 % (0) | 2.4 % (1) | 17.6 % (1) | 20.3 % (3) | 16.6 % (1) | 16.1 % (0) | 24.5 % (2) | 11.8 % (1) | 22.5 % (4) |
| whisper-cpp-tiny | greedy (beam 1) | 3.7 % (0) | 1.6 % (0) | 16.2 % (1) | 27.0 % (3) | 25.5 % (2) | 20.0 % (4) | 23.5 % (2) | 16.2 % (1) | 33.3 % (5) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 4.9 % (1) | 6.8 % (1) | 9.5 % (2) | 6.9 % (2) | 7.3 % (1) | 12.5 % (0) | 1.5 % (0) | 17.1 % (4) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 4.9 % (1) | 14.9 % (2) | 9.5 % (2) | 11.0 % (2) | 9.8 % (1) | 15.5 % (1) | 10.3 % (0) | 18.9 % (5) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 1.6 % (1) | 5.4 % (1) | 1.4 % (0) | 3.4 % (1) | 3.4 % (1) | 7.0 % (0) | 0.0 % (0) | 6.3 % (2) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 1.9 % (1) | 2.4 % (1) | 5.4 % (1) | 1.4 % (0) | 3.4 % (1) | 3.9 % (1) | 8.5 % (0) | 0.0 % (0) | 7.2 % (2) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `fr-med-09-owner` (median WER 70 %): reference "L'ordonnance mentionne de l'ibuprofène 400 mg en cas de douleur."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "L'ordonnance mentionne de l'ibuprofène 400 mg en cas de douleur."
- `fr-it-18-owner` (median WER 43 %): reference "Le service redémarre automatiquement en cas d'erreur."; best output (sherpa-canary-180m-flash-int8, greedy search): "Le service redémarre automatiquement en cas d'erreur."
- `fr-it-02-owner` (median WER 38 %): reference "Peux-tu redémarrer le cluster Kubernetes avant midi ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Peux-tu redémarrer le cluster Kubernetes avant midi?"
- `en-it-05-owner` (median WER 33 %): reference "Open port 443 on the firewall."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Open port 543 on the firewall."
- `fr-adm-06-owner` (median WER 33 %): reference "Les frais de dossier s'élèvent à 120 francs suisses."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Les frais de dossier s'élèvent à 120 francs suisses."
- `fr-gen-q-12-owner` (median WER 33 %): reference "Quelle est la météo prévue pour demain à Lausanne ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Quelle est la météo prévue pour demain à Lausanne?"
- `fr-med-05-owner` (median WER 33 %): reference "L'anticoagulant est contre-indiqué en cas de saignement actif."; best output (sherpa-canary-180m-flash-int8, greedy search): "L'anticoagulant est contre - indiqué en cas de saignement actif."
- `fr-it-03-owner` (median WER 30 %): reference "L'API REST renvoie une erreur 500 quand le token expire."; best output (sherpa-canary-180m-flash-int8, greedy search): "L'API reste renvoie une erreur cinq cents quand le token expire."
- `fr-it-01-owner` (median WER 27 %): reference "La base de données PostgreSQL ne répond plus depuis ce matin."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "La base de données PostgreSQL ne répond plus depuis ce matin."
- `fr-gen-q-02-owner` (median WER 25 %): reference "Peux-tu ouvrir le dossier de madame Dupont ?"; best output (sherpa-canary-180m-flash-int8, greedy search): "Peux - tu ouvrir le dossier de Madame Dupont?"

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / en-gen-s-03-owner: Number expected `(none)` found `9`: a number was added
- sherpa-whisper-tiny / greedy search / en-gen-s-06-owner: Negation expected `1 negation marker(s)` found `2`: a negation was added: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-whisper-tiny / greedy search / en-it-05-owner: KeyTerm expected `firewall` found `fire wall`: expected tech term not found; closest text is 100 % similar (probable misspelling)
- sherpa-whisper-tiny / greedy search / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-adm-04-owner: Date expected `mardi` found `mars`: a day or month was changed
- sherpa-whisper-tiny / greedy search / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Number expected `443` found `(absent)`: a number was lost
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Unit expected `(none)` found `h`: a unit was added
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Unit expected `(none)` found `l`: a unit was added
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: KeyTerm expected `firewall` found `(absent)`: expected tech term not found
- sherpa-canary-180m-flash-int8 / greedy search / fr-med-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-canary-180m-flash-int8 / greedy search / fr-med-09-owner: KeyTerm expected `ibuprofène` found `ibuprophène`: expected drug term not found; closest text is 82 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-med-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-med-08-owner: Number expected `10` found `1`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / en-it-05-owner: KeyTerm expected `firewall` found `fire wall`: expected tech term not found; closest text is 100 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-04-owner: Date expected `mardi` found `mars`: a day or month was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / en-it-05-owner: KeyTerm expected `firewall` found `fire wall`: expected tech term not found; closest text is 100 % similar (probable misspelling)
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-04-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-adm-02-owner: Number expected `2026` found `2016`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-it-03-owner: Number expected `(none)` found `10`: a number was added
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-it-03-owner: KeyTerm expected `token` found `(absent)`: expected tech term not found
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-adm-02-owner: Number expected `2026` found `2016`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-med-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-med-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
