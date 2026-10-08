# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 95 | 0 | 5.1 % | 5.7 % | 2.9 % | 1 | 7 | 9 | 0.150 | 726 | 1224 | 2271 | 1877 | 8.6 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | en-general | en-technical | fr-administrative | fr-dictation | fr-general | fr-with-english-terms | fr-legal | fr-medical |
|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 1.9 % (1) | 18.7 % (2) | 5.4 % (1) | 1.4 % (0) | 7.6 % (1) | 2.9 % (0) | 1.5 % (1) | 1.5 % (0) | 3.6 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `en-gen-q-05-owner` (median WER 100 %): reference "Is the meeting room free at 3 30?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "PostgreSQL REST API REST API Thirty?"
- `en-gen-q-01-owner` (median WER 67 %): reference "What time is my next appointment?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "PostgreSQL a next appointment?"
- `en-gen-s-02-owner` (median WER 62 %): reference "The letter is ready, you can send it."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "REST API can send it"
- `en-gen-s-04-owner` (median WER 50 %): reference "I will not be available on Monday afternoon."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "REST API available on Monday afternoon."
- `fr-gen-q-09-owner` (median WER 25 %): reference "Qui m'a appelé ce matin avant 10 heures ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Qui m'appelé ce matin avant 10 heures?"

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-dict-01-owner: Number expected `12000` found `121000`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-gen-q-05-owner: Number expected `330` found `30`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-gen-s-04-owner: Negation expected `1 negation marker(s)` found `0`: a negation was dropped: the meaning may be reversed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: Negation expected `2 negation marker(s)` found `1`: a negation was dropped: the meaning may be reversed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension artériel`: expected term term not found; closest text is 88 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-it-03-owner: Unit expected `(none)` found `l`: a unit was added
