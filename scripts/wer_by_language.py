#!/usr/bin/env python3
"""Aggregate view of a benchmark results folder: micro WER per language with a bootstrap interval,
critical-severity sample counts, and key terms found per kind.

Usage: python -I -X utf8 scripts/wer_by_language.py benchmark/results/<run folder> [--resamples 5000] [--seed 7]

Same method as scripts/bootstrap_ci.py (repetition 1 only, sentences resampled with replacement,
micro WER = total errors / total reference words, 95 % interval = 2.5th to 97.5th percentile),
applied separately to the French and the English samples. Only aggregates are printed (no texts),
so the script is safe on a private run, but the usual rule applies: never paste from the private
folder (I-047).

Key terms: a key term counts as found when the detector raised no `keyTerm` flag for it in that
sample. The number of occurrences per kind is the largest number of flags seen for that kind in a
single configuration plus the number found there, so it is reconstructed from the flags and the
result is cross-checked against `bench termstudy` (same counts: drug 6, term 10, tech 34, legal 6
on the public dataset).
"""
import argparse
import json
import random
from collections import defaultdict
from pathlib import Path


def percentile(sorted_values, p):
    k = max(0, min(len(sorted_values) - 1, int(round(p / 100 * (len(sorted_values) - 1)))))
    return sorted_values[k]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("folder", type=Path)
    ap.add_argument("--resamples", type=int, default=5000)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--terms", type=Path, default=None,
                    help="folder with the sample JSON files (default: benchmark/samples next to the results)")
    args = ap.parse_args()

    rows = defaultdict(list)
    for line in (args.folder / "runs.jsonl").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r["repetition"] != 1 or r.get("error"):
            continue
        rows[(r["modelId"], r["decoding"])].append(r)

    rng = random.Random(args.seed)
    print(f"{'configuration':58s} {'lang':4s} {'n':>3s} {'words':>5s} {'WER':>6s} {'95 % CI':>15s} {'crit':>4s}")
    for key in sorted(rows):
        for lang in ("fr", "en"):
            rs = [r for r in rows[key] if r["language"] == lang]
            if not rs:
                continue
            errors = lambda sel: sum(r["substitutions"] + r["deletions"] + r["insertions"] for r in sel)
            words = lambda sel: sum(r["referenceWords"] for r in sel)
            dist = []
            for _ in range(args.resamples):
                d = [rng.choice(rs) for _ in rs]
                dist.append(errors(d) / words(d))
            dist.sort()
            crit = sum(1 for r in rs if r["hasCritical"])
            print(f"{key[0] + ' | ' + key[1]:58s} {lang:4s} {len(rs):3d} {words(rs):5d} "
                  f"{errors(rs) / words(rs):6.1%} [{percentile(dist, 2.5):5.1%},{percentile(dist, 97.5):5.1%}] {crit:4d}")

    samples_dir = args.terms or (args.folder.parent.parent / "samples")
    occurrences = defaultdict(int)
    kinds_by_sample = {}
    for f in samples_dir.glob("*.json"):
        s = json.loads(f.read_text(encoding="utf-8"))
        kinds_by_sample[s["id"]] = s.get("keyTerms", [])
    any_config = next(iter(rows.values()))
    for r in any_config:
        for t in kinds_by_sample.get(r["sampleId"], []):
            occurrences[t["kind"]] += 1

    print()
    print("Key terms found / occurrences, all samples, repetition 1")
    kinds = sorted(occurrences)
    print(f"{'configuration':58s} " + " ".join(f"{k:>8s}" for k in kinds))
    print(f"{'(occurrences in the dataset)':58s} " + " ".join(f"{occurrences[k]:8d}" for k in kinds))
    for key in sorted(rows):
        missed = defaultdict(int)
        for r in rows[key]:
            for c in r["critical"]:
                if c["kind"] == "keyTerm":
                    # the flag text starts with "expected <kind> term not found"
                    for k in kinds:
                        if c["message"].startswith(f"expected {k} term"):
                            missed[k] += 1
        print(f"{key[0] + ' | ' + key[1]:58s} " +
              " ".join(f"{occurrences[k] - missed[k]:>3d}/{occurrences[k]:<4d}" for k in kinds))


if __name__ == "__main__":
    main()
