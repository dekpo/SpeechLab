# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Chunking: clips longer than 25 s were cut at silences (Silero VAD) and the texts joined; shorter clips were passed whole. Inference time is the sum over the segments; speech detection is extra (`chunkingMs` in runs.jsonl).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3 | 0 | 21.1 % | 20.0 % | 8.2 % | 0 | 2 | 8 | 0.109 | 3033 | 3905 | 505 | 672 | 6.9 |
| sherpa-canary-180m-flash-int8 | greedy search | 3 | 0 | 2.0 % | 2.1 % | 1.7 % | 0 | 1 | 3 | 0.186 | 5350 | 6684 | 1344 | 777 | 5.4 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 3 | 0 | 0.5 % | 0.4 % | 0.1 % | 0 | 1 | 1 | 0.139 | 3559 | 5259 | 2070 | 1556 | 5.5 |
| whisper-cpp-tiny | beam search (5 beams) | 3 | 0 | 16.6 % | 16.2 % | 6.9 % | 0 | 2 | 5 | 0.134 | 3477 | 4816 | 388 | 193 | 3.8 |
| whisper-cpp-tiny | greedy (beam 1) | 3 | 0 | 24.6 % | 23.6 % | 9.6 % | 0 | 2 | 9 | 0.100 | 2772 | 3582 | 427 | 164 | 3.6 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 3 | 0 | 10.1 % | 10.2 % | 4.2 % | 0 | 3 | 7 | 0.252 | 7406 | 10302 | 379 | 215 | 3.9 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 3 | 0 | 11.1 % | 11.2 % | 4.7 % | 0 | 3 | 7 | 0.207 | 6072 | 8092 | 353 | 167 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 3 | 0 | 4.0 % | 4.0 % | 0.9 % | 0 | 2 | 3 | 0.881 | 25890 | 35544 | 828 | 480 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 3 | 0 | 5.5 % | 5.8 % | 1.6 % | 0 | 2 | 4 | 0.819 | 24067 | 30172 | 833 | 361 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | fr-dictation |
|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3.7 % (0) | 27.6 % (2) |
| sherpa-canary-180m-flash-int8 | greedy search | 1.9 % (0) | 2.1 % (1) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 0.0 % (0) | 0.7 % (1) |
| whisper-cpp-tiny | beam search (5 beams) | 5.6 % (0) | 20.7 % (2) |
| whisper-cpp-tiny | greedy (beam 1) | 5.6 % (0) | 31.7 % (2) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 13.1 % (2) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 14.5 % (2) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 4.8 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 7.4 % (1) | 4.8 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

None.

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `12` found `245`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `mars` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `0`: a negation was dropped: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: KeyTerm expected `attestation` found `(absent)`: expected term term not found
- sherpa-canary-180m-flash-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `(absent)`: expected term term not found
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `9` found `2.45`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `(absent)`: expected term term not found
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxicillin`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `(absent)`: expected term term not found
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxicillin`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `(absent)`: expected term term not found
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: KeyTerm expected `PostgreSQL` found `post gress`: expected tech term not found; closest text is 80 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
