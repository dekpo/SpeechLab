# Benchmark summary

Machine: Intel(R) Core(TM) 7 150U (12 logical cores), 23.6 GB RAM, Windows 11 Home x86_64, build release, none (CPU only).

Accuracy uses repetition 1 only. WER micro = total errors / total reference words; macro = mean of per-sample WER. Critical = samples with at least one critical flag (changed number, unit, negation, weekday/month, missing key term). Memory is a sampled lower bound. One speaker only: these are not general claims.

## Overall

| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 95 | 0 | 2.7 % | 2.7 % | 0.8 % | 0 | 3 | 7 | 0.142 | 710 | 1012 | 2204 | 1877 | 8.8 |

## WER micro by category (critical samples in brackets)

| Model | Decoding | en-dictation | en-general | en-technical | fr-administrative | fr-dictation | fr-general | fr-with-english-terms | fr-legal | fr-medical |
|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-parakeet-tdt-0.6b-v3-int8 | greedy search | 0.0 % (0) | 2.4 % (0) | 5.4 % (1) | 1.4 % (0) | 3.4 % (1) | 2.9 % (0) | 2.5 % (0) | 1.5 % (0) | 2.7 % (1) |

## Hard or suspect samples (median WER over all configurations >= 25 %)

If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.

- `fr-it-03-owner` (median WER 30 %): reference "L'API REST renvoie une erreur 500 quand le token expire."; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "La PI reste, renvoie une erreur 500 quand le token expire."
- `fr-gen-q-09-owner` (median WER 25 %): reference "Qui m'a appelé ce matin avant 10 heures ?"; best output (sherpa-parakeet-tdt-0.6b-v3-int8, greedy search): "Qui m'appelé ce matin avant 10 heures?"

## Critical flags to review (repetition 1, first 6 per configuration)

- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / en-it-05-owner: Number expected `443` found `543`: a number was changed
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `amoxicilline` found `amoxycilline`: expected drug term not found; closest text is 92 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-dict-01-owner: KeyTerm expected `tension artérielle` found `tension artériel`: expected term term not found; closest text is 88 % similar (probable misspelling)
- sherpa-parakeet-tdt-0.6b-v3-int8 / greedy search / fr-med-08-owner: Number expected `10` found `1`: a number was changed
