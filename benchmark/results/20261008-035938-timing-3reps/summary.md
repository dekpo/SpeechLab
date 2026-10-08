# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 40 | 0 | 25.9 % | 26.0 % | 18.7 % | 3 | 5 | 6 | 0.059 | 266 | 399 | 440 | 682 | 7.5 |
| sherpa-canary-180m-flash-int8 | greedy search | 40 | 0 | 1.5 % | 1.6 % | 0.8 % | 0 | 0 | 0 | 0.095 | 414 | 528 | 1017 | 802 | 7.4 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 40 | 0 | 2.7 % | 2.7 % | 1.0 % | 0 | 0 | 0 | 0.099 | 440 | 557 | 1776 | 1585 | 10.4 |
| whisper-cpp-tiny | beam search (5 beams) | 40 | 0 | 11.0 % | 10.6 % | 3.9 % | 0 | 1 | 1 | 0.143 | 608 | 696 | 86 | 193 | 3.7 |
| whisper-cpp-tiny | greedy (beam 1) | 40 | 0 | 13.1 % | 13.2 % | 5.5 % | 0 | 4 | 5 | 0.120 | 517 | 549 | 87 | 164 | 3.5 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 40 | 0 | 6.4 % | 6.4 % | 2.5 % | 0 | 2 | 2 | 0.313 | 1356 | 1497 | 75 | 215 | 3.8 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 40 | 0 | 7.9 % | 7.6 % | 2.9 % | 0 | 2 | 2 | 0.293 | 1272 | 1373 | 79 | 167 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 40 | 0 | 2.7 % | 2.6 % | 1.0 % | 0 | 2 | 2 | 1.318 | 5696 | 6329 | 198 | 481 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 40 | 0 | 3.4 % | 3.2 % | 1.2 % | 0 | 2 | 2 | 1.256 | 5373 | 5943 | 200 | 361 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-general | fr-general |
|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 36.6 % (2) | 19.5 % (3) |
| sherpa-canary-180m-flash-int8 | greedy search | 1.6 % (0) | 1.5 % (0) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 2.4 % (0) | 2.9 % (0) |
| whisper-cpp-tiny | beam search (5 beams) | 2.4 % (1) | 16.1 % (0) |
| whisper-cpp-tiny | greedy (beam 1) | 1.6 % (0) | 20.0 % (4) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 4.9 % (1) | 7.3 % (1) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 4.9 % (1) | 9.8 % (1) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.6 % (1) | 3.4 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 2.4 % (1) | 3.9 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `fr-gen-q-12-owner` (median WER 33 %): reference "Quelle est la météo prévue pour demain à Lausanne ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Quelle est la météo prévue pour demain à Lausanne?"
- `fr-gen-q-02-owner` (median WER 25 %): reference "Peux-tu ouvrir le dossier de madame Dupont ?"; best output (sherpa-canary-180m-flash-int8, greedy search): "Peux - tu ouvrir le dossier de Madame Dupont?"

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / en-gen-s-03-owner: Number expected `(none)` found `9`: a number was added
- sherpa-whisper-tiny / greedy search / en-gen-s-06-owner: Negation expected `1 negation marker(s)` found `2`: a negation was added: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / fr-gen-q-03-owner: Unit expected `h` found `(absent)`: a unit was lost
- sherpa-whisper-tiny / greedy search / fr-gen-q-09-owner: Number expected `10` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-gen-q-09-owner: Unit expected `h` found `(absent)`: a unit was lost
- sherpa-whisper-tiny / greedy search / fr-gen-q-10-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-gen-q-03-owner: Unit expected `h` found `(absent)`: a unit was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-gen-q-07-owner: Number expected `(none)` found `15`: a number was added
- whisper-cpp-tiny / greedy (beam 1) / fr-gen-q-09-owner: Number expected `10` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-gen-q-09-owner: Unit expected `h` found `(absent)`: a unit was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-gen-q-10-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-gen-s-08-owner: Unit expected `(none)` found `m`: a unit was added
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-gen-s-08-owner: Unit expected `(none)` found `m`: a unit was added
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-gen-q-05-owner: Number expected `330` found `3.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-gen-s-03-owner: Number expected `(none)` found `0`: a number was added
