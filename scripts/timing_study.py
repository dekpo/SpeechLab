#!/usr/bin/env python3
"""Timing study of a benchmark folder that has several repetitions per sample.

Usage: python scripts/timing_study.py <results folder>

Reads runs.jsonl and prints, per configuration (model + decoding):
- inference time over ALL repetitions: median, p95, min, max (ms) and the real-time factor median;
- run-to-run variation of the same sample: coefficient of variation (standard deviation / mean)
  across the repetitions of one sample; the median and the maximum over samples are shown;
- the median inference time of each repetition (repetition 1 is the cold one for sherpa-onnx,
  whose model stays loaded between repetitions; whisper.cpp starts a new process every time, so
  every repetition is cold);
- the number of samples whose transcript differs between repetitions (should be 0: deterministic);
- peak memory and busy cores (median).
Only successful runs are used. Nothing is invented: a value that cannot be computed shows n/a.
"""
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def pct(values, p):
    values = sorted(values)
    if not values:
        return None
    k = (len(values) - 1) * p / 100.0
    lo, hi = int(k), min(int(k) + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def fmt(v, digits=0):
    return "n/a" if v is None else f"{v:.{digits}f}"


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    folder = Path(sys.argv[1])
    cfg_runs = defaultdict(list)
    failures = 0
    for line in (folder / "runs.jsonl").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r.get("error"):
            failures += 1
            continue
        cfg_runs[(r["modelId"], r["decoding"])].append(r)
    print(f"{sum(len(v) for v in cfg_runs.values())} successful runs, {failures} failures\n")

    print("| Configuration | n | infer median ms | p95 | min | max | RTF median | CV median % | CV max % |")
    print("|---|---|---|---|---|---|---|---|---|")
    detail = []
    for cfg, runs in sorted(cfg_runs.items()):
        inf = [r["inferenceMs"] for r in runs]
        by_sample = defaultdict(list)
        for r in runs:
            by_sample[r["sampleId"]].append(r)
        cvs = []
        for rs in by_sample.values():
            vals = [x["inferenceMs"] for x in rs]
            if len(vals) >= 2 and statistics.mean(vals) > 0:
                cvs.append(100.0 * statistics.pstdev(vals) / statistics.mean(vals))
        name = f"{cfg[0]} / {cfg[1]}"
        print(
            f"| {name} | {len(runs)} | {fmt(statistics.median(inf))} | {fmt(pct(inf, 95))} | {fmt(min(inf))} | "
            f"{fmt(max(inf))} | {fmt(statistics.median(r['rtf'] for r in runs), 3)} | "
            f"{fmt(statistics.median(cvs) if cvs else None, 1)} | {fmt(max(cvs) if cvs else None, 1)} |"
        )
        reps = sorted({r["repetition"] for r in runs})
        per_rep = {k: statistics.median(r["inferenceMs"] for r in runs if r["repetition"] == k) for k in reps}
        unstable = [s for s, rs in by_sample.items() if len({x["text"].strip() for x in rs}) > 1]
        detail.append((name, per_rep, unstable, runs))

    print("\n| Configuration | median inference per repetition (ms) | samples with changing transcript | peak memory MB (median) | busy cores (median) |")
    print("|---|---|---|---|---|")
    for name, per_rep, unstable, runs in detail:
        reps_txt = ", ".join(f"rep {k}: {v:.0f}" for k, v in per_rep.items())
        print(
            f"| {name} | {reps_txt} | {len(unstable)} | "
            f"{statistics.median(r['peakMemoryMb'] for r in runs):.0f} | "
            f"{statistics.median(r['avgCores'] for r in runs):.1f} |"
        )
        for s in unstable[:5]:
            print(f"|   unstable: {s} | | | | |")

    # sherpa-onnx records the model load on the first sample only (other records show 0), so the
    # cold load is the largest value; whisper.cpp reloads in every process, so all records count.
    print("\nModel load time, ms (median of records with a load, maximum):")
    for name, _per_rep, _u, runs in detail:
        loads = [r["loadMs"] for r in runs if r["loadMs"] > 0]
        print(f"  {name}: median {fmt(statistics.median(loads) if loads else None)}, max {fmt(max(loads) if loads else None)}")


if __name__ == "__main__":
    main()
