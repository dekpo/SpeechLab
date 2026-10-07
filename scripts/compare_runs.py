#!/usr/bin/env python3
"""Compare two benchmark result folders: determinism of the transcripts and change in speed.

Usage: python scripts/compare_runs.py <old folder> <new folder>

Both folders must contain runs.jsonl. Repetition 1 of every (model, decoding, sample) is compared.

Part 1, ACCURACY / DETERMINISM: how many transcripts are identical, how many differ, and for the
differing ones the WER of both runs. Engines are expected to be deterministic for the same input,
model and thread count, so any difference must be explained (not hidden).
Part 2, SPEED: median real-time factor, inference time, cold load time, peak memory and busy cores
per configuration, old versus new. If the new run was made on a quiet machine and the old one was
disturbed, the NEW figures are the ones to quote.
"""
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def load(folder):
    runs = {}
    for line in (Path(folder) / "runs.jsonl").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r["repetition"] == 1:
            runs[(r["modelId"], r["decoding"], r["sampleId"])] = r
    return runs


def med(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def fmt(v, digits=2):
    return "n/a" if v is None else f"{v:.{digits}f}"


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    old, new = load(sys.argv[1]), load(sys.argv[2])
    keys = sorted(set(old) & set(new))
    only_old, only_new = set(old) - set(new), set(new) - set(old)
    print(f"{len(keys)} runs in common; only in old: {len(only_old)}; only in new: {len(only_new)}\n")

    print("== Part 1: transcripts (determinism)")
    by_cfg = defaultdict(list)
    for k in keys:
        by_cfg[(k[0], k[1])].append(k)
    total_diff = 0
    for cfg, ks in sorted(by_cfg.items()):
        diffs = [k for k in ks if old[k]["text"].strip() != new[k]["text"].strip()]
        total_diff += len(diffs)
        print(f"{cfg[0][:38]:38s} {cfg[1][:22]:22s} identical {len(ks) - len(diffs):3d}/{len(ks):3d}")
        for k in diffs[:5]:
            print(f"    DIFF {k[2]}: WER {fmt(old[k]['wer'], 3)} -> {fmt(new[k]['wer'], 3)}")
            print(f"      old: {old[k]['text'][:110]}")
            print(f"      new: {new[k]['text'][:110]}")
    print(f"\ntotal differing transcripts: {total_diff} of {len(keys)}")
    print("(expected 0 for a deterministic engine; investigate before publishing if not)\n")

    print("== Part 2: speed, memory, cores (median over samples; old -> new)")
    print(f"{'configuration':62s} {'RTF':>15s} {'infer ms':>15s} {'load ms':>13s} {'peak MB':>13s} {'cores':>11s}")
    for cfg, ks in sorted(by_cfg.items()):
        def col(field, digits, source):
            return med([source[k][field] for k in ks if source[k].get("error") is None])
        row = []
        for field, digits in (("rtf", 3), ("inferenceMs", 0), ("loadMs", 0), ("peakMemoryMb", 0), ("avgCores", 1)):
            a, b = col(field, digits, old), col(field, digits, new)
            row.append(f"{fmt(a, digits)}->{fmt(b, digits)}")
        name = f"{cfg[0][:36]} | {cfg[1][:20]}"
        print(f"{name:62s} {row[0]:>15s} {row[1]:>15s} {row[2]:>13s} {row[3]:>13s} {row[4]:>11s}")


if __name__ == "__main__":
    main()
