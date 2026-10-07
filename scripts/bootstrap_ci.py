#!/usr/bin/env python3
"""Bootstrap confidence intervals for the micro WER of every configuration in a results folder.

Usage: python scripts/bootstrap_ci.py benchmark/results/<run folder> [--resamples 5000] [--seed 7]
       [--exclude sample-id,sample-id]

Method: samples (sentences) are resampled with replacement; for each resample the micro WER is
(total errors) / (total reference words) over the drawn samples. The 95 % interval is the 2.5th to
97.5th percentile. A paired comparison against the best configuration resamples the SAME samples
for both, so the interval of the WER difference is meaningful: if it excludes 0, the gap is
unlikely to be an artefact of which sentences were drawn. One speaker and ~95 sentences: this
measures sampling noise only, NOT speaker, microphone or accent variation.
"""
import argparse
import json
import random
from collections import defaultdict
from pathlib import Path


def load(folder, exclude):
    runs = defaultdict(dict)  # config -> sample -> (errors, words)
    for line in (folder / "runs.jsonl").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r["repetition"] != 1 or r.get("error") or r["sampleId"] in exclude:
            continue
        key = f'{r["modelId"]} | {r["decoding"]}'
        runs[key][r["sampleId"]] = (r["substitutions"] + r["deletions"] + r["insertions"], r["referenceWords"])
    return runs


def percentile(sorted_values, p):
    k = max(0, min(len(sorted_values) - 1, int(round(p / 100 * (len(sorted_values) - 1)))))
    return sorted_values[k]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("folder", type=Path)
    ap.add_argument("--resamples", type=int, default=5000)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--exclude", default="")
    args = ap.parse_args()
    exclude = {s for s in args.exclude.split(",") if s}
    runs = load(args.folder, exclude)
    samples = sorted(set.intersection(*(set(v) for v in runs.values())))
    rng = random.Random(args.seed)
    draws = [[rng.choice(samples) for _ in samples] for _ in range(args.resamples)]

    def wer(config, idx):
        e = sum(runs[config][s][0] for s in idx)
        w = sum(runs[config][s][1] for s in idx)
        return e / w if w else 0.0

    full = {c: wer(c, samples) for c in runs}
    best = min(full, key=full.get)
    print(f"{len(samples)} samples in common, {args.resamples} resamples, seed {args.seed}"
          + (f", excluded: {sorted(exclude)}" if exclude else ""))
    print(f"{'configuration':64s} {'WER':>6s}  {'95 % CI':>15s}   vs best ({best.split(' | ')[0]})")
    for c in sorted(runs, key=full.get):
        dist = sorted(wer(c, d) for d in draws)
        diff = sorted(wer(c, d) - wer(best, d) for d in draws)
        lo, hi = percentile(diff, 2.5), percentile(diff, 97.5)
        verdict = "-" if c == best else ("clearly worse" if lo > 0 else "not distinguishable")
        print(f"{c:64s} {full[c]:6.1%}  [{percentile(dist, 2.5):5.1%}, {percentile(dist, 97.5):5.1%}]   "
              f"{'' if c == best else f'diff {full[c]-full[best]:+.1%} [{lo:+.1%}, {hi:+.1%}] {verdict}'}")


if __name__ == "__main__":
    main()
