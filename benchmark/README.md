# Benchmark dataset

Reproducible test material for the speech-to-text evaluation. Everything here is data: samples are
added by dropping files in folders, never by changing code.

> The sentences are invented test material. They mention drugs, doses and legal terms only to test
> recognition. They are **not** medical or legal advice and contain no real patient data.

## Layout

```
benchmark/
  scripts/            sentences to read aloud (committed)
  samples/            one JSON per recorded sample: metadata + reference transcript (committed)
  samples-private/    same format, for material that must never be committed (git-ignored)
  audio/              the WAV files (git-ignored: they contain voices)
  results/            benchmark outputs (committed, except results/private/)
```

Override the root with `SPEECHLAB_BENCH_DIR`.

## Scripts (what to read)

| File | Items | Content |
|---|---|---|
| `fr-general.json` | 24 | French voice queries (12 questions) and statements (12) for an office assistant |
| `fr-professional.json` | 24 | French medical (10), administrative (7) and legal (7) vocabulary, with numbers, units, dates, negations |
| `fr-it.json` | 20 | French sentences containing English technical terms (PostgreSQL, Kubernetes, API REST, Symfony...) |
| `en-general.json` | 16 | English questions (8) and statements (8) |
| `en-technical.json` | 8 | English technical sentences |
| `dictation.json` | 3 | Long dictations (about 1 minute): two French, one English. Optional |

A script item:

```json
{
  "id": "fr-med-01",
  "language": "fr",
  "domain": "medical",
  "utteranceType": "statement",
  "category": "fr-medical",
  "text": "Le patient prend 500 mg d'amoxicilline trois fois par jour pendant 7 jours.",
  "keyTerms": [{ "text": "amoxicilline", "kind": "drug" }]
}
```

`text` is the exact reference transcript. `keyTerms` are words that must be recognised; kinds:
`drug` and `name` (a miss is a critical error), `legal`, `tech`, `term` (a miss is a warning).
Values: language `fr|en`; domain `general|medical|administrative|legal|it`; utteranceType
`question|statement|dictation`.

## Samples (what was recorded)

```json
{
  "id": "fr-med-01-owner",
  "language": "fr",
  "domain": "medical",
  "utteranceType": "statement",
  "category": "fr-medical",
  "speaker": { "id": "owner", "profile": "native French, no marked accent", "gender": null, "accent": null },
  "audioFile": "fr-med-01-owner.wav",
  "reference": "Le patient prend 500 mg d'amoxicilline trois fois par jour pendant 7 jours.",
  "keyTerms": [{ "text": "amoxicilline", "kind": "drug" }],
  "source": { "kind": "own-recording", "url": null, "license": "own recording, owner consent" },
  "committable": false,
  "notes": ""
}
```

- `audioFile` is a file name inside `audio/` (16 kHz mono 16-bit WAV is what the app produces; other
  WAV formats work too). Duration is measured automatically.
- `source.kind`: `own-recording`, `third-party-private`, `public-domain`, `licensed`, `synthetic`.
  `source.license` is mandatory. Public sources need `source.url`.
- `speaker.profile` is an honest description of ONE speaker. One speaker never represents an accent:
  reports say "this speaker", not "Swiss French".
- `committable` says whether the audio may be shared. Default false.

## Privacy rules

- Voices are personal data. `audio/` and `samples-private/` are git-ignored.
- Recordings of other people (voice notes) are `third-party-private`: they must live in
  `samples-private/`, are never committed, never quoted in documents, and only aggregate numbers are
  reported. Ask the speaker for consent whenever possible.
- Clips downloaded from the internet need a source URL and a licence; without a clear licence they stay
  local (`samples-private/`).
- No real patient data, ever.

## Importing an existing recording (private clips, accents)

`bench import` adds an audio file you already have, with the reference text you typed yourself:

```
bench import --wav clip.wav --id acc-01 --reference-file acc-01.txt --speaker spk-a \
             --accent "Swiss-Romande" --gender female --private --consent yes
```

- Audio: any PCM WAV; it is converted to 16 kHz mono 16-bit and written to `audio/<id>.wav`. m4a, mp3
  or Ogg files must be converted to WAV first.
- Metadata goes to `samples-private/<id>.json` with `source.kind` = `third-party-private`; `--consent`
  must be `yes` or `unknown` (written into the licence field); `no` refuses the import.
  Without `--private`, give `--source-kind`, `--license` and (public domain or licensed) `--url`.
- The reference file is read and never printed. It is the ground truth: nobody corrects it.
- Describe the speaker with `--profile` or `--accent` / `--gender`. It describes this one speaker; it
  is not an accent label. Default category: `fr-accent-private` (`--category` to change it).
- An existing id is refused unless `--replace`. After an import, `bench check` lists the clip.
- Run the clips with `bench run --include-private --category fr-accent-private --label <name>`; results
  go to `results/private/`. Those files contain the texts of the clips: never copy them into a document;
  report only counts and error rates and say "this speaker".

## Text-to-speech material (M6)

`tts/sentences-fr.txt`, `tts/sentences-en.txt` (taken from the public scripts above) feed `tts measure`;
`tts/listening/*.txt` are the texts of the listening samples. Generated audio goes to `tts-samples/`
(git-ignored, local listening only). `results/<stamp>-tts-m6/` holds timings only, no audio.

## Recording guidelines

- One sentence per clip, natural pace, quiet room, same microphone distance.
- Check the level message after each recording: peak under 100 % and no clipping.
- Read numbers as you would say them ("500 mg" can be "cinq cents milligrammes"): the scoring folds
  spoken numbers and unit spellings to the same form, but keeps different values different.
- Read exactly the text. If you stumble, record the sentence again rather than editing the reference.

## What the scoring does (and does not)

- WER and CER on lightly normalised text (case, punctuation, hyphens; spoken numbers and unit
  spellings folded). Originals are kept.
- Separate critical-error flags: changed, lost or added numbers; changed units; dropped or added
  negations; changed weekday or month; missing key terms (with "probable misspelling" hints).
  These are heuristics for evaluation, not a safety guarantee.

## Vocabulary files (drug-name study, D-036)

`vocab/fr.txt` and `vocab/en.txt`: one term per line, `#` for comments. They hold the key terms of the dataset plus a few common drug names (names only). Because the key terms come from the test sentences themselves, results obtained with these files are an upper bound on what a vocabulary known in advance can do. Used by `bench run --vocab-dir`, `bench postcorrect` and the engines' biasing (never on by default).
