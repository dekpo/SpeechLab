# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 95 | 0 | 26.5 % | 28.1 % | 16.7 % | 5 | 18 | 43 | 0.077 | 365 | 578 | 490 | 894 | 7.4 |
| sherpa-canary-180m-flash-int8 | greedy search | 95 | 0 | 8.0 % | 10.4 % | 3.5 % | 1 | 1 | 7 | 0.111 | 536 | 903 | 1147 | 1087 | 7.4 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 95 | 0 | 5.9 % | 6.0 % | 2.4 % | 0 | 2 | 4 | 0.114 | 551 | 785 | 1956 | 1875 | 9.4 |
| whisper-cpp-tiny | beam search (5 beams) | 95 | 0 | 17.5 % | 17.7 % | 7.1 % | 0 | 11 | 28 | 0.159 | 762 | 915 | 104 | 242 | 3.7 |
| whisper-cpp-tiny | greedy (beam 1) | 95 | 0 | 21.2 % | 20.6 % | 9.0 % | 1 | 17 | 44 | 0.129 | 625 | 674 | 103 | 213 | 3.5 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 95 | 0 | 10.2 % | 10.6 % | 4.2 % | 0 | 10 | 18 | 0.337 | 1626 | 1910 | 88 | 265 | 3.9 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 95 | 0 | 13.2 % | 13.3 % | 5.1 % | 0 | 13 | 28 | 0.293 | 1426 | 1626 | 88 | 216 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 95 | 0 | 7.2 % | 7.1 % | 3.0 % | 0 | 5 | 10 | 1.236 | 5881 | 6837 | 199 | 533 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 95 | 0 | 7.7 % | 7.6 % | 3.4 % | 0 | 5 | 10 | 1.117 | 5403 | 6070 | 199 | 410 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | en-general | en-technical | fr-administrative | fr-dictation | fr-general | fr-with-english-terms | fr-legal | fr-medical |
|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 0.0 % (0) | 36.6 % (2) | 28.4 % (1) | 23.0 % (3) | 30.3 % (2) | 22.9 % (3) | 27.0 % (2) | 26.5 % (1) | 29.7 % (4) |
| sherpa-canary-180m-flash-int8 | greedy search | 1.9 % (0) | 1.6 % (0) | 45.9 % (1) | 6.8 % (0) | 5.5 % (0) | 4.9 % (0) | 5.5 % (0) | 5.9 % (0) | 8.1 % (0) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 1.9 % (0) | 2.4 % (0) | 9.5 % (1) | 6.8 % (0) | 4.8 % (0) | 6.3 % (0) | 5.0 % (0) | 7.4 % (0) | 9.9 % (1) |
| whisper-cpp-tiny | beam search (5 beams) | 1.9 % (0) | 2.4 % (1) | 17.6 % (1) | 24.3 % (3) | 20.0 % (1) | 19.5 % (0) | 24.0 % (2) | 16.2 % (1) | 18.9 % (2) |
| whisper-cpp-tiny | greedy (beam 1) | 1.9 % (0) | 1.6 % (0) | 16.2 % (1) | 28.4 % (3) | 29.7 % (2) | 23.4 % (4) | 24.0 % (2) | 20.6 % (1) | 30.6 % (4) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 3.7 % (1) | 4.9 % (1) | 10.8 % (1) | 13.5 % (2) | 11.0 % (2) | 10.7 % (1) | 12.0 % (0) | 7.4 % (0) | 13.5 % (2) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 3.7 % (1) | 4.9 % (1) | 17.6 % (2) | 13.5 % (2) | 15.2 % (2) | 13.2 % (1) | 16.0 % (1) | 13.2 % (0) | 16.2 % (3) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 3.7 % (1) | 1.6 % (1) | 9.5 % (1) | 5.4 % (0) | 7.6 % (0) | 6.8 % (1) | 9.5 % (0) | 5.9 % (0) | 11.7 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 3.7 % (1) | 2.4 % (1) | 9.5 % (1) | 5.4 % (0) | 7.6 % (0) | 7.3 % (1) | 10.5 % (0) | 5.9 % (0) | 12.6 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `fr-med-09-owner` (median WER 70 %): reference "L'ordonnance mentionne de l'ibuprofène 400 mg en cas de douleur."; best output (sherpa-canary-180m-flash-int8, greedy search): "L'ordonnance mentionne de l'ibuprofène quatre cents milligrammes en cas de douleur."
- `fr-it-18-owner` (median WER 43 %): reference "Le service redémarre automatiquement en cas d'erreur."; best output (sherpa-canary-180m-flash-int8, greedy search): "Le service redémarre automatiquement en cas d'erreur."
- `fr-gen-q-05-owner` (median WER 38 %): reference "Combien de patients sont inscrits cet après-midi ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Combien de patients sont inscrits cet API REST-midi?"
- `fr-it-02-owner` (median WER 38 %): reference "Peux-tu redémarrer le cluster Kubernetes avant midi ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Peux-tu redémarrer le cluster Kubernetes avant midi?"
- `fr-it-01-owner` (median WER 36 %): reference "La base de données PostgreSQL ne répond plus depuis ce matin."; best output (sherpa-canary-180m-flash-int8, greedy search): "La base de données PostgreSQL ne répond plus depuis ce main."
- `en-it-05-owner` (median WER 33 %): reference "Open port 443 on the firewall."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Open port 543 on the firewall."
- `fr-adm-06-owner` (median WER 33 %): reference "Les frais de dossier s'élèvent à 120 francs suisses."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Les frais de dossier s'élèvent à 120 francs suisses."
- `fr-gen-q-12-owner` (median WER 33 %): reference "Quelle est la météo prévue pour demain à Lausanne ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Quelle est la météo prévue pour demain à Lausanne?"
- `fr-it-03-owner` (median WER 30 %): reference "L'API REST renvoie une erreur 500 quand le token expire."; best output (sherpa-canary-180m-flash-int8, greedy search): "L'API REST renvoie une erreur cinq cents quand le token expire."
- `fr-gen-q-02-owner` (median WER 25 %): reference "Peux-tu ouvrir le dossier de madame Dupont ?"; best output (sherpa-canary-180m-flash-int8, greedy search): "Peux - tu ouvrir le dossier de Madame Dupont?"
- `fr-gen-s-07-owner` (median WER 25 %): reference "Je ne serai pas disponible lundi après-midi."; best output (sherpa-canary-180m-flash-int8, greedy search): "Je ne serai pas disponible lundi API REST - midi."

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / en-gen-s-03-owner: Number expected `(none)` found `9`: a number was added
- sherpa-whisper-tiny / greedy search / en-gen-s-06-owner: Negation expected `1 negation marker(s)` found `2`: a negation was added: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-adm-04-owner: Date expected `mardi` found `mars`: a day or month was changed
- sherpa-whisper-tiny / greedy search / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Number expected `443` found `(absent)`: a number was lost
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Unit expected `(none)` found `h`: a unit was added
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: Unit expected `(none)` found `l`: a unit was added
- sherpa-canary-180m-flash-int8 / greedy search / en-it-05-owner: KeyTerm expected `firewall` found `(absent)`: expected tech term not found
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-med-08-owner: Number expected `10` found `1`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-02-owner: Number expected `2026` found `2020`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-04-owner: Date expected `mardi` found `mars`: a day or month was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-adm-07-owner: Number expected `2045` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
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
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-med-09-owner: Number expected `(none)` found `4`: a number was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-it-05-owner: Number expected `443` found `543`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-med-09-owner: Number expected `(none)` found `4`: a number was added
