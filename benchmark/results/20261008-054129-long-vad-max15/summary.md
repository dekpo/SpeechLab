# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Chunking: clips longer than 25 s were cut at silences (Silero VAD) and the texts joined; shorter clips were passed whole. Inference time is the sum over the segments; speech detection is extra (`chunkingMs` in runs.jsonl).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 3 | 0 | 21.1 % | 19.4 % | 6.8 % | 0 | 2 | 10 | 0.104 | 3198 | 3748 | 469 | 744 | 5.8 |
| sherpa-canary-180m-flash-int8 | greedy search | 3 | 0 | 1.5 % | 1.6 % | 0.6 % | 0 | 1 | 2 | 0.234 | 6491 | 8403 | 1087 | 800 | 5.1 |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 3 | 0 | 3.0 % | 2.9 % | 1.6 % | 0 | 1 | 3 | 0.137 | 4279 | 4926 | 2494 | 1577 | 4.6 |
| whisper-cpp-tiny | beam search (5 beams) | 3 | 0 | 16.6 % | 15.8 % | 5.8 % | 0 | 1 | 8 | 0.108 | 3031 | 3886 | 368 | 195 | 3.9 |
| whisper-cpp-tiny | greedy (beam 1) | 3 | 0 | 17.6 % | 16.7 % | 7.0 % | 0 | 2 | 9 | 0.065 | 2199 | 2328 | 308 | 164 | 3.6 |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 3 | 0 | 7.5 % | 7.5 % | 3.0 % | 0 | 3 | 7 | 0.188 | 6183 | 6584 | 269 | 217 | 4.0 |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 3 | 0 | 9.5 % | 9.5 % | 4.4 % | 0 | 3 | 5 | 0.147 | 5098 | 5283 | 278 | 168 | 3.8 |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 3 | 0 | 2.5 % | 2.4 % | 0.6 % | 0 | 2 | 2 | 0.695 | 21886 | 23522 | 603 | 481 | 3.9 |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 3 | 0 | 4.0 % | 4.2 % | 1.4 % | 0 | 2 | 3 | 0.528 | 18208 | 18985 | 622 | 361 | 3.9 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | fr-dictation |
|---|---|---|---|
| sherpa-whisper-tiny | greedy search | 1.9 % (0) | 28.3 % (2) |
| sherpa-canary-180m-flash-int8 | greedy search | 1.9 % (0) | 1.4 % (1) |
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 1.9 % (0) | 3.4 % (1) |
| whisper-cpp-tiny | beam search (5 beams) | 5.6 % (0) | 20.7 % (1) |
| whisper-cpp-tiny | greedy (beam 1) | 5.6 % (0) | 22.1 % (2) |
| whisper-cpp-base-q5_1 | beam search (5 beams) | 1.9 % (1) | 9.7 % (2) |
| whisper-cpp-base-q5_1 | greedy (beam 1) | 1.9 % (1) | 12.4 % (2) |
| whisper-cpp-small-q5_1 | beam search (5 beams) | 1.9 % (1) | 2.8 % (1) |
| whisper-cpp-small-q5_1 | greedy (beam 1) | 7.4 % (1) | 2.8 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

None.

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `12` found `149`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `14` found `245`: a number was changed
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `9` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-01-owner: Date expected `mars` found `(absent)`: a day or month was lost
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- sherpa-whisper-tiny / greedy search / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- sherpa-canary-180m-flash-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: Number expected `10` found `1`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: Unit expected `h` found `(absent)`: a unit was lost
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `9` found `245`: a number was changed
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxicyline`: expected drug term not found; closest text is 83 % similar (probable misspelling)
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `14` found `149`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `9` found `2.45`: a number was changed
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Number expected `2045` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `jeudi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-01-owner: Date expected `mardi` found `(absent)`: a day or month was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-tiny / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxicillin`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / beam search (5 beams) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-base-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-01-owner: Number expected `7` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Number expected `5` found `(absent)`: a number was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: Date expected `janvier` found `(absent)`: a day or month was lost
- whisper-cpp-base-q5_1 / greedy (beam 1) / fr-dict-02-owner: KeyTerm expected `attestation` found `testation`: expected term term not found; closest text is 82 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / beam search (5 beams) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / beam search (5 beams) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: Number expected `1030` found `10.30`: a number was changed
- whisper-cpp-small-q5_1 / greedy (beam 1) / en-dict-01-owner: KeyTerm expected `PostgreSQL` found `post gress`: expected tech term not found; closest text is 80 % similar (probable misspelling)
- whisper-cpp-small-q5_1 / greedy (beam 1) / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxiciline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
