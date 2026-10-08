# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3 | 0 | 20.1 % | 18.0 % | 9.5 % | 0 | 2 | 11 | 0.090 | 3236 | 4029 | 512 | 922 | 5.1 |
| sherpa-canary-180m-flash-int8 | greedy search | 3 | 0 | 1.5 % | 1.8 % | 1.0 % | 0 | 0 | 1 | 0.471 | 12110 | 17658 | 1354 | 1015 | 4.2 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 3 | 0 | 0.5 % | 0.4 % | 0.1 % | 0 | 1 | 1 | 0.165 | 4849 | 6130 | 2713 | 1809 | 4.1 |
| whisper-cpp-tiny | beam search (5 beams) | 3 | 0 | 12.1 % | 11.0 % | 4.4 % | 0 | 1 | 5 | 0.092 | 2356 | 3734 | 107 | 244 | 4.0 |
| whisper-cpp-tiny | greedy (beam 1) | 3 | 0 | 19.6 % | 18.4 % | 7.3 % | 0 | 2 | 9 | 0.052 | 1338 | 2127 | 112 | 213 | 3.7 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 3 | 0 | 5.5 % | 5.4 % | 2.2 % | 0 | 3 | 5 | 0.173 | 4782 | 6216 | 93 | 265 | 4.0 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 3 | 0 | 8.5 % | 8.4 % | 3.4 % | 0 | 3 | 6 | 0.106 | 3124 | 5347 | 120 | 216 | 3.7 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 3 | 0 | 3.0 % | 2.8 % | 0.7 % | 0 | 2 | 2 | 0.478 | 12292 | 18910 | 240 | 532 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 3 | 0 | 3.0 % | 2.8 % | 0.7 % | 0 | 2 | 2 | 0.293 | 7660 | 14067 | 218 | 410 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | fr-dictation |
|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 1.9 % (0) | 26.9 % (2) |
| sherpa-canary-180m-flash-int8 | greedy search | 3.7 % (0) | 0.7 % (0) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 0.0 % (0) | 0.7 % (1) |
| whisper-cpp-tiny | beam search (5 beams) | 0.0 % (0) | 16.6 % (1) |
| whisper-cpp-tiny | greedy (beam 1) | 3.7 % (0) | 25.5 % (2) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 6.9 % (2) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 11.0 % (2) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 3.4 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 1.9 % (1) | 3.4 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

None.

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `9` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `30` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `45`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
