# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Chunking: clips longer than 25 s were cut at silences (Silero VAD) and the texts joined; shorter clips were passed whole. Inference time is the sum over the segments; speech detection is extra (`chunkingMs` in runs.jsonl).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3 | 0 | 18.1 % | 16.9 % | 6.8 % | 0 | 2 | 9 | 0.114 | 3575 | 4085 | 532 | 942 | 5.4 |
| sherpa-canary-180m-flash-int8 | greedy search | 3 | 0 | 6.5 % | 7.9 % | 7.5 % | 0 | 1 | 3 | 0.263 | 9304 | 9469 | 1282 | 938 | 4.5 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 3 | 0 | 0.5 % | 0.4 % | 0.1 % | 0 | 1 | 1 | 0.134 | 4146 | 4530 | 2347 | 1708 | 4.7 |
| whisper-cpp-tiny | beam search (5 beams) | 3 | 0 | 15.1 % | 14.6 % | 5.6 % | 0 | 2 | 7 | 0.089 | 2330 | 3414 | 201 | 197 | 3.9 |
| whisper-cpp-tiny | greedy (beam 1) | 3 | 0 | 20.6 % | 20.1 % | 8.0 % | 0 | 2 | 9 | 0.052 | 1730 | 1882 | 221 | 165 | 3.7 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 3 | 0 | 7.0 % | 6.9 % | 2.7 % | 0 | 3 | 7 | 0.155 | 4805 | 5568 | 192 | 218 | 4.0 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 3 | 0 | 7.5 % | 7.3 % | 3.0 % | 0 | 3 | 6 | 0.101 | 3626 | 3736 | 171 | 169 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 3 | 0 | 1.5 % | 1.6 % | 0.3 % | 0 | 2 | 2 | 0.475 | 15163 | 17058 | 380 | 485 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 3 | 0 | 2.5 % | 2.4 % | 0.6 % | 0 | 2 | 2 | 0.387 | 13153 | 13895 | 418 | 362 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | fr-dictation |
|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3.7 % (0) | 23.4 % (2) |
| sherpa-canary-180m-flash-int8 | greedy search | 22.2 % (1) | 0.7 % (0) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 0.0 % (0) | 0.7 % (1) |
| whisper-cpp-tiny | beam search (5 beams) | 5.6 % (0) | 18.6 % (2) |
| whisper-cpp-tiny | greedy (beam 1) | 5.6 % (0) | 26.2 % (2) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 9.0 % (2) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 9.7 % (2) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 1.4 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 1.9 % (1) | 2.8 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

None.

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `0`: a negation was dropped: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- sherpa-canary-180m-flash-int8 / greedy search / en-dict-01-owner: Unit expected `h` found `(absent)`: a unit was lost
- sherpa-canary-180m-flash-int8 / greedy search / en-dict-01-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- sherpa-canary-180m-flash-int8 / greedy search / en-dict-01-owner: KeyTerm expected `PostgreSQL` found `postgress`: expected tech term not found; closest text is 80 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
